package prover

import (
	"crypto/sha256"
	"fmt"
	"math/big"
	"sync"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/witness"
)

type ProverManager struct {
	proverMap sync.Map
}

const chunkSize = fr.Bytes

// Create a prover in manager, which is identified by projectID and is constructed with circuit
// encoded in binary and proving key encoded in provingKey.
func (p *ProverManager) NewProject(projectID string, projectVersion string, binary []byte, provingKey []byte) error {
	if _, exist := p.proverMap.Load(projectKey(projectID, projectVersion)); exist {
		return nil
	}

	prover := &Groth16Prover{}
	if err := prover.LoadCircuit(binary); err != nil {
		return fmt.Errorf("failed to load project %s circuit: %w", projectID, err)
	}

	if err := prover.LoadProvingKey(provingKey); err != nil {
		return fmt.Errorf("failed to load project %s proving key: %w", projectID, err)
	}

	p.proverMap.Store(projectKey(projectID, projectVersion), prover)

	return nil
}

func projectKey(projectID string, projectVersion string) [32]byte {
	data := projectID + projectVersion
	return sha256.Sum256([]byte(data))
}

// Execute a proving task on prover identified by projectID with payloads as witness.
// The encoding of witness is conducted according
// to https://docs.gnark.consensys.io/HowTo/serialize#witness. The output Exec is the
// calldata of verifyproof func in verifier contract.
func (p *ProverManager) Exec(projectID string, projectVersion string, payloads []byte) ([]byte, error) {
	prover, exist := p.proverMap.Load(projectKey(projectID, projectVersion))
	if !exist {
		return nil, fmt.Errorf("project %s does not exist", projectID)
	}

	wit, _ := witness.New(ecc.BN254.ScalarField())
	wit.UnmarshalBinary(payloads)

	proof, err := prover.(*Groth16Prover).Prove(wit)
	if err != nil {
		return nil, fmt.Errorf("failed to prove project %s: %w", projectID, err)
	}

	return encodeProofForSolidity(proof, wit)
}

func encodeProofForSolidity(proof groth16.Proof, wit witness.Witness) ([]byte, error) {
	proofSol, ok := proof.(interface{ MarshalSolidity() []byte })
	if !ok {
		return nil, fmt.Errorf("failed to marshal proof to solidity")
	}
	proofBytes := proofSol.MarshalSolidity()

	commitmentsBytes := []byte{}
	switch {
	case len(proofBytes) < 8*chunkSize:
		return nil, fmt.Errorf("invalid proof length")
	case len(proofBytes) == 8*chunkSize:
		commitmentsBytes = []byte{}
	case len(proofBytes) > 8*chunkSize:
		commitmentsBytes = proofBytes[8*chunkSize:]
		commitmentCount := new(big.Int).SetBytes(commitmentsBytes[:4]).Int64()
		if len(commitmentsBytes) < int(4+(commitmentCount+2)*chunkSize) {
			return nil, fmt.Errorf("invalid commitments length")
		}
		commitmentsBytes = commitmentsBytes[4:]
	}
	pubwit, err := wit.Public()
	if err != nil {
		return nil, err
	}
	witnessBytes, err := pubwit.MarshalBinary()
	if err != nil {
		return nil, err
	}
	return append(proofBytes[:8*chunkSize], append(commitmentsBytes, witnessBytes[12:]...)...), nil
}
