package prover_test

import (
	"bytes"
	"crypto/sha256"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/std/math/uints"
	"github.com/iotexproject/w3bstream-vm/gnark-server/prover"
	"github.com/stretchr/testify/require"
)

type addCircuit struct {
	X uints.U32
	Y uints.U32 `gnark:",public"`
}

func (circuit *addCircuit) Define(api frontend.API) error {
	uapi, err := uints.New[uints.U32](api)
	if err != nil {
		return err
	}
	uapi.AssertEq(circuit.X, circuit.Y)
	return nil
}

func setupaddCircuit(t *testing.T) ([]byte, []byte, witness.Witness, groth16.VerifyingKey) {
	// Compile circuit
	var mc addCircuit
	r1cs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &mc)
	require.NoError(t, err)

	// Generate proving key
	pk, vk, err := groth16.Setup(r1cs)
	require.NoError(t, err)

	// Create witness
	assignment := &addCircuit{
		X: uints.NewU32(0x12345678),
		Y: uints.NewU32(0x12345678),
	}
	witness, err := frontend.NewWitness(assignment, ecc.BN254.ScalarField())
	require.NoError(t, err)

	// Serialize circuit and proving key
	var circuitBuf, pkBuf bytes.Buffer
	_, err = r1cs.WriteTo(&circuitBuf)
	require.NoError(t, err)
	_, err = pk.WriteTo(&pkBuf)
	require.NoError(t, err)

	return circuitBuf.Bytes(), pkBuf.Bytes(), witness, vk
}

func TestGroth16Prover(t *testing.T) {
	circuitData, pkData, witness, vk := setupaddCircuit(t)

	p := &prover.Groth16Prover{}

	t.Run("load circuit", func(t *testing.T) {
		err := p.LoadCircuit(circuitData)
		require.NoError(t, err)
	})

	t.Run("load proving key", func(t *testing.T) {
		err := p.LoadProvingKey(pkData)
		require.NoError(t, err)
	})

	t.Run("generate proof", func(t *testing.T) {
		proof, err := p.Prove(witness)
		require.NoError(t, err)
		require.NotNil(t, proof)

		publicWitness, err := witness.Public()
		require.NoError(t, err)

		err = groth16.Verify(proof, vk, publicWitness, backend.WithVerifierHashToFieldFunction(sha256.New()))
		require.NoError(t, err)
	})
}
