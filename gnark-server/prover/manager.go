package prover

import (
	"fmt"
	"sync"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
)

type ProverManager struct {
	proverMap sync.Map
}

// Create a prover in manager, which is identified by projectID and is constructed with circuit
// encoded in binary and proving key encoded in provingKey.
func (p *ProverManager) NewProject(projectID uint64, binary []byte, provingKey []byte) error {
	if _, exist := p.proverMap.Load(projectID); exist {
		return fmt.Errorf("project %d already exists", projectID)
	}

	prover := &Groth16Prover{}
	if err := prover.LoadCircuit(binary); err != nil {
		return fmt.Errorf("failed to load project %d circuit: %w", projectID, err)
	}

	if err := prover.LoadProvingKey(provingKey); err != nil {
		return fmt.Errorf("failed to load project %d proving key: %w", projectID, err)
	}

	p.proverMap.Store(projectID, prover)

	return nil
}

// Execute a proving task on prover identified by projectID with payloads as witness.
// The encoding of witness is conducted according
// to https://docs.gnark.consensys.io/HowTo/serialize#witness. The output is the proof is TBD
func (p *ProverManager) Exec(projectID uint64, payloads []byte) ([]byte, error) {
	prover, exist := p.proverMap.Load(projectID)
	if !exist {
		return nil, fmt.Errorf("project %d does not exist", projectID)
	}

	wit, _ := witness.New(ecc.BN254.ScalarField())
	wit.UnmarshalBinary(payloads)

	proof, err := prover.(*Groth16Prover).Prove(wit)
	if err != nil {
		return nil, fmt.Errorf("failed to prove project %d: %w", projectID, err)
	}

	proofSol, ok := proof.(interface{ MarshalSolidity() []byte })
	if !ok {
		return nil, fmt.Errorf("failed to marshal proof to solidity")
	}

	// TODO: return abiencoded proof
	return proofSol.MarshalSolidity(), nil
}
