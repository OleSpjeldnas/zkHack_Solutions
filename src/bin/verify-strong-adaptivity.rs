#![allow(unused, unreachable_code)]
use ark_ed_on_bls12_381::Fr;
use ark_ff::Field;
use strong_adaptivity::{Instance, Proof, data::puzzle_data};
use strong_adaptivity::verify;
use strong_adaptivity::PUZZLE_DESCRIPTION;
use prompt::{puzzle, welcome};
use ark_ec::{AffineCurve, ProjectiveCurve};
use ark_ed_on_bls12_381::{EdwardsAffine as GAffine};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError};
use ark_std::{UniformRand, io::{Read, Write}};
use rand::Rng;
use strong_adaptivity::ProofResponse;
use strong_adaptivity::utils::b2s_hash_to_field;
use strong_adaptivity::ProofCommitment;


fn main() {
    welcome();
    puzzle(PUZZLE_DESCRIPTION);
    let ck = puzzle_data();

    let (instance, witness, proof): (Instance, (Fr, Fr, Fr, Fr), Proof) = {
        let rng = &mut rand::thread_rng();
        let r_rho = Fr::rand(rng);
        let r_tau = Fr::rand(rng);
        let (comm_rho, rho) = ck.commit_with_rng(r_rho, rng);
        let (comm_tau, tau) = ck.commit_with_rng(r_tau, rng);
        let commitment = ProofCommitment {
            comm_rho,
            comm_tau,
        };
    
        let challenge = b2s_hash_to_field(&(ck, commitment));
        let a = Fr::rand(rng);
        let b = (r_rho-r_tau+challenge*a)/challenge;
        let ra = Fr::rand(rng);
        let rb = Fr::rand(rng);

        let s = r_rho + challenge * a;
        let u = rho + challenge * ra;
        let t = tau + challenge * rb;
        let response = ProofResponse { s, u, t };

    (Instance {comm_1: ck.commit_with_explicit_randomness(a, ra), comm_2: ck.commit_with_explicit_randomness(b, rb)},
    (a, ra, b, rb),
    Proof {
        commitment,
        response,
    }
    )


    };
    
    let (a_1, r_1, a_2, r_2) = witness;

    assert!(verify(&ck, &instance, &proof));
    // Check that commitments are correct
    assert_eq!(ck.commit_with_explicit_randomness(a_1, r_1), instance.comm_1);
    assert_eq!(ck.commit_with_explicit_randomness(a_2, r_2), instance.comm_2);
    // Check that messages are unequal
    assert_ne!(a_1, a_2);
}