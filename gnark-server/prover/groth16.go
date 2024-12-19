package prover

import (
	"bytes"
	"crypto/sha256"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/std"
	"github.com/consensys/gnark/std/math/uints"
)

type Groth16Prover struct {
	circuit    constraint.ConstraintSystem
	provingKey groth16.ProvingKey
}

func init() {
	std.RegisterHints()
}

func (p *Groth16Prover) LoadCircuit(b []byte) error {
	cs := groth16.NewCS(ecc.BN254)
	_, err := cs.ReadFrom(bytes.NewReader(b))
	if err != nil {
		return err
	}
	p.circuit = cs
	return nil
}

func (p *Groth16Prover) LoadProvingKey(b []byte) error {
	pk := groth16.NewProvingKey(ecc.BN254)
	_, err := pk.ReadFrom(bytes.NewReader(b))
	if err != nil {
		return err
	}
	p.provingKey = pk
	return nil
}

func (p *Groth16Prover) Prove(wit witness.Witness) (groth16.Proof, error) {
	return groth16.Prove(p.circuit, p.provingKey, wit,
		backend.WithProverHashToFieldFunction(sha256.New()),
		backend.WithSolverOptions(solver.WithHints(getAllHints()...)),
	)
}

func getAllHints() []solver.Hint {
	hints := [][]solver.Hint{
		uints.GetHints(),
	}

	allHints := make([]solver.Hint, 0)
	for _, h := range hints {
		allHints = append(allHints, h...)
	}
	return allHints
}
