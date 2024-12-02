package prover_test

import (
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/iotexproject/w3bstream-vm/gnark-server/prover"
	"github.com/stretchr/testify/require"
)

func TestProverManager(t *testing.T) {
	circuitData, pkData, wit, _ := setupaddCircuit(t)
	manager := &prover.ProverManager{}

	t.Run("new project", func(t *testing.T) {
		// Test creating a new project
		err := manager.NewProject("1", "v1", circuitData, pkData)
		require.NoError(t, err)

		// Test creating a duplicate project
		err = manager.NewProject("1", "v1", circuitData, pkData)
		require.NoError(t, err)
	})

	t.Run("exec project", func(t *testing.T) {
		// Create a project first
		err := manager.NewProject("2", "v1", circuitData, pkData)
		require.NoError(t, err)

		// Marshal the witness to bytes
		witBytes, err := wit.MarshalBinary()
		require.NoError(t, err)

		// Execute the proving task
		proof, err := manager.Exec("2", "v1", witBytes)
		require.NoError(t, err)
		require.NotEmpty(t, proof)
	})

	t.Run("exec non-existent project", func(t *testing.T) {
		wit, _ := witness.New(ecc.BN254.ScalarField())
		witBytes, _ := wit.MarshalBinary()

		// Try to execute a non-existent project
		_, err := manager.Exec("999", "v1", witBytes)
		require.Error(t, err)
	})

	t.Run("exec with invalid witness", func(t *testing.T) {
		// Create a project
		err := manager.NewProject("3", "v1", circuitData, pkData)
		require.NoError(t, err)

		// Create an invalid witness
		invalidAssignment := &addCircuit{
			X: 42,
			Y: 100, // This should be 84 (2*42)
		}
		invalidWitness, err := frontend.NewWitness(invalidAssignment, ecc.BN254.ScalarField())
		require.NoError(t, err)

		// Marshal the invalid witness
		invalidWitBytes, err := invalidWitness.MarshalBinary()
		require.NoError(t, err)

		// Execute with invalid witness should fail
		_, err = manager.Exec("3", "v1", invalidWitBytes)
		require.Error(t, err)
	})
}
