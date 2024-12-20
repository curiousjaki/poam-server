use std::collections::HashMap;
use rules::event_filter::InsertEvent;
use operations::Operation;
use rules::conformance::{PoamInput, PoamMetadata, RuleInput};
use rules::{CardinalityRule, PrecedenceRule, Rule};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use methods::{
    COMPOSITE_PROVING_ELF, COMPOSITE_PROVING_ID, VERIFIABLE_PROCESSING_ELF,
    VERIFIABLE_PROCESSING_ID,
};
use qfilter::Filter;
use anyhow::Error;
use once_cell::sync::Lazy;

static ELF_MAP: Lazy<HashMap<[u32; 8], &[u8]>> = Lazy::new(|| {
    HashMap::from([
        (VERIFIABLE_PROCESSING_ID, VERIFIABLE_PROCESSING_ELF),
        (COMPOSITE_PROVING_ID, COMPOSITE_PROVING_ELF),
    ])
});


pub fn prove_method(
    method_payload: &String,
    pi: &PoamInput,
    previous_receipt: Option<Receipt>,
) -> Receipt {
    println!("Build Proof and send to vm");
    //let method_payload = operations::OperationRequest { a, b, operation };
    let ser_pi: String =
        serde_json::to_string(&pi).unwrap();
    let mut env_builder = ExecutorEnv::builder();
    match previous_receipt {
        Some(receipt) => {
            env_builder.add_assumption(receipt);
        }
        None => {}
    }

    let env = env_builder
        .write(&method_payload)
        .unwrap()
        .write(&ser_pi)
        .unwrap()
        .build()
        .unwrap();
    // read the input
    let elf = ELF_MAP.get(&pi.image_id).unwrap();
    let prover = default_prover();
    let prove_info = prover.prove(env, elf).unwrap();
    return prove_info.receipt;
}

//pub fn perform_composite_prove(receipts: Vec<Receipt>, image_id: [u32; 8]) -> Result<Receipt, Error> {
//    let mut env_builder = ExecutorEnv::builder();
//    let mut cpi: Vec<CompositeProofInput> = Vec::new();
//    for r in receipts.iter() {
//        //println!("{:?}",r.metadata);
//        env_builder.add_assumption(r.clone());
//        cpi.push(CompositeProofInput{image_id:image_id.clone(),public_data: r.journal.decode().unwrap()});
//    }
//    let cpi_string = serde_json::to_string(&cpi).unwrap();
//    println!("{:?}",cpi_string);
//    let env = env_builder
//        .write(&cpi_string)
//        .unwrap()
//        .build()
//        .unwrap();
//
//    let prover = default_prover();
//    // read the input
//    let prove = prover.prove(env, COMPOSITE_PROVING_ELF);
//    match prove {
//        Ok(prove_info) => {
//        Ok(prove_info) => {
//            return Ok(prove_info.receipt);
//        }
//        Err(e) => {
//            return Err(e);
//        }
//    }
//}


#[cfg(test)]
mod tests {
    use operations::OperationRequest;

    use super::*;

   //RISC0_DEV_MODE=0 RUST_LOG=info cargo test --release -- --nocapture
    #[test]
    fn test_proving_method(){
        println!("Starting the Program");
        //env_logger::init();
        // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`

        //tracing_subscriber::fmt()
        //    .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        //    .init();
        //let filter = qfilter::Filter::new(1000, 0.01).expect("Failed to create filter");
        //let rule1 = Rule::Cardinality(CardinalityRule{prior: [1,2,3,4,5,6,7,8],max: 1, min: 1});
        //let rule_set: RuleSet = RuleSet{rules: vec![rule1], qf: filter};
        //let mut qf = Filter::new(100, 0.01)
        //    .expect("Failed to create filter");
        //qf.insert_event(VERIFIABLE_PROCESSING_ID).unwrap();
        
        let rules1: Vec<Rule> = vec![Rule::Precedence(PrecedenceRule {
        //current: VERIFIABLE_PROCESSING_ID,
        preceeding: VERIFIABLE_PROCESSING_ID,
        })];

        let method_payload1 = serde_json::to_string(
            &OperationRequest{a: 1.0, b: 2.0, operation: Operation::Add })
            .unwrap();

        let pi1: PoamInput = PoamInput {
            image_id: VERIFIABLE_PROCESSING_ID,
            rule_input: RuleInput {
                //current_image_id: VERIFIABLE_PROCESSING_ID,
                rules: None,
                ordering_rules: None,
            },
            public_data: None,
        };

        let receipt1 = prove_method(
            &method_payload1,
            &pi1,None);
        //&receipt1.verify(cm.current_image_id).unwrap();
        let (result_json,metadata_json):(String,String) = receipt1.journal.decode().unwrap();
        println!("Result: {}, Metadata: {}",result_json, metadata_json);

        let pi2: PoamInput = PoamInput {
            image_id: VERIFIABLE_PROCESSING_ID,
            rule_input: RuleInput {
                //current_image_id: VERIFIABLE_PROCESSING_ID,
                rules: Some(rules1),
                ordering_rules: None,
            },
            public_data: Some((result_json,metadata_json)),
        };
        let receipt2 = prove_method(
            &method_payload1,
            &pi2,Some(receipt1));
        //&receipt1.verify(cm.current_image_id).unwrap();
        let (result_json2,metadata_json2):(String,String) = receipt2.journal.decode().unwrap();
        println!("Result: {}, Metadata: {}",result_json2, metadata_json2);

        //let receipts: Vec<Receipt> = vec![receipt1]; //, receipt2, receipt3, receipt4];//, receipt3, receipt4];
        //println!("Receipt vector created");
        //let composite_receipt = perform_composite_prove(receipts, VERIFIABLE_PROCESSING_ID)
        //    .expect("Failed to prove composite receipt");
        // TODO: Implement code for retrieving receipt journal here.

        // The receipt was verified at the end of proving, but the below code is an
        // example of how someone else could verify this receipt.
        //println!("Composite receipt created");
        //composite_receipt.verify(COMPOSITE_PROVING_ID).unwrap();
    }
}