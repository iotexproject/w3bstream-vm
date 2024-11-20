package prover

import (
	"bytes"
	"crypto/sha256"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
)

type Groth16Prover struct {
	circuit    constraint.ConstraintSystem
	provingKey groth16.ProvingKey
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
	return groth16.Prove(p.circuit, p.provingKey, wit, backend.WithProverHashToFieldFunction(sha256.New()))
}
