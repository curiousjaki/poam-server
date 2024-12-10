use qfilter::Filter;
use rules::{CardinalityRule, ConformanceMetadata, InsertEvent, PrecedenceRule, Rule};
use serde_json;
use std::any::Any;
use std::str::FromStr;
use operations::Operation;
// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use anyhow::Error;
use methods::{
    COMPOSITE_PROVING_ELF, COMPOSITE_PROVING_ID, VERIFIABLE_PROCESSING_ELF,
    VERIFIABLE_PROCESSING_ID,
};
use host::{prove_method,perform_composite_prove};
use proto::verifiable_processing_service_server::{
    VerifiableProcessingService, VerifiableProcessingServiceServer,
};
use proto::{
    CompositionRequest, CompositionResponse, Proof, ProveRequest, ProveResponse, VerifyRequest,
    VerifyResponse,
};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use tonic::{transport::Server, Request, Response, Status};
pub mod proto {
    tonic::include_proto!("poam");
}

#[derive(Default)]
pub struct VerifiableProcessingServiceServerImplementation;

#[tonic::async_trait]
impl VerifiableProcessingService for VerifiableProcessingServiceServerImplementation {
    async fn prove(
        &self,
        request: Request<ProveRequest>,
    ) -> Result<Response<ProveResponse>, Status> {
        //println!("Got a request: {:?}", request);

        let request = request.into_inner();
        let a = request.variable_a;
        let b = request.variable_b;
        let operation = request.operation;

        let receipt = prove_method(
            a,
            b,
            Operation::from_str(&operation).unwrap(),
            &ConformanceMetadata {
                previous_image_id: VERIFIABLE_PROCESSING_ID,
                current_image_id: VERIFIABLE_PROCESSING_ID,
                rules: vec![],
                qf: Filter::new(100, 0.01).unwrap(),
            },
        );

        let (response_value, qfilter_json): (f64, String) =
            receipt.journal.decode::<(f64, String)>().unwrap();
        //let response_2 = receipt.journal.decode().unwrap();

        let filter: qfilter::Filter = serde_json::from_str(&qfilter_json).unwrap();
        println!("\n filter: {:?}", &filter);
        //println!("{:?}",filter);
        let reply = ProveResponse {
            //receipt: Some(receipt.into()),
            response_value: response_value,
            proof_response: Some(Proof {
                image_id: VERIFIABLE_PROCESSING_ID.to_vec(),
                receipt: bincode::serialize(&receipt).unwrap(),
            }),
            proof_chain: vec![Proof {
                image_id: VERIFIABLE_PROCESSING_ID.to_vec(),
                receipt: bincode::serialize(&receipt).unwrap(),
            }],
        };
        Ok(Response::new(reply))
    }

    async fn compose(
        &self,
        request: Request<CompositionRequest>,
    ) -> Result<Response<CompositionResponse>, Status> {
        let request = request.into_inner();
        let receipts: Vec<Receipt> = request
            .proof_chain
            .iter()
            .map(|proof| {
                let receipt: Receipt = bincode::deserialize(&proof.receipt).unwrap();
                receipt
            })
            .collect();

        let composite_receipt = perform_composite_prove(receipts, VERIFIABLE_PROCESSING_ID)
            .expect("Failed to prove composite receipt");

        // TODO: Implement code for retrieving receipt journal here.

        let reply = CompositionResponse {
            proof_response: Some(Proof {
                image_id: vec![0],
                receipt: bincode::serialize(&composite_receipt).unwrap(),
            }),
            proof_chain: vec![Proof {
                image_id: vec![0],
                receipt: vec![0],
            }],
        };
        Ok(Response::new(reply))
    }

    async fn verify(
        &self,
        request: Request<VerifyRequest>,
    ) -> Result<Response<VerifyResponse>, Status> {
        let request = request.into_inner();
        let proof = request
            .proof
            .ok_or(Status::invalid_argument("Missing proof"))?;
        let image_id: [u32; 8] = proof
            .image_id
            .try_into()
            .expect("Failed to convert Vec<u32> to [u32; 8]");
        let receipt: Receipt = bincode::deserialize(&proof.receipt).unwrap();
        //print_type_of(&req.receipt);
        //let receipt = Receipt::from(request.receipt.unwrap());
        let verification_result = receipt.verify(image_id);

        //println!("{:?}",receipt.journal.);
        let reply: VerifyResponse;
        match verification_result {
            Ok(_) => {
                reply = VerifyResponse {
                    is_valid_executed: true,
                    public_output: "public_output".to_string(),
                };
            }
            Err(err) => {
                reply = VerifyResponse {
                    is_valid_executed: false,
                    public_output: "public_output".to_string(),
                };
                println!("{:?}", err)
            }
        }
        Ok(Response::new(reply))
    }
}



fn main() {
    //env_logger::init();
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    println!("Started the Program");
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
    //let filter = qfilter::Filter::new(1000, 0.01).expect("Failed to create filter");
    //let rule1 = Rule::Cardinality(CardinalityRule{prior: [1,2,3,4,5,6,7,8],max: 1, min: 1});
    //let rule_set: RuleSet = RuleSet{rules: vec![rule1], qf: filter};
    let mut f = Filter::new(100, 0.01).expect("Failed to create filter");
    f.insert_event(VERIFIABLE_PROCESSING_ID);
    let mut rules: Vec<Rule> = vec![Rule::Precedence(PrecedenceRule {
        current: VERIFIABLE_PROCESSING_ID,
        preceeding: VERIFIABLE_PROCESSING_ID,
    })];
    let cm: ConformanceMetadata = ConformanceMetadata {
        previous_image_id: VERIFIABLE_PROCESSING_ID,
        current_image_id: VERIFIABLE_PROCESSING_ID,
        rules: rules,
        qf: f,
    };

    let receipt1 = prove_method(1.0, 2.0, Operation::Add, &cm);
    //let receipt2 = prove_method(1.0, 2.0, Operation::Mul, rule_set.clone());
    //let receipt3 = prove_method(1.0, 2.0, Operation::Sub, rule_set.clone());
    //let receipt4 = prove_method(1.0, 2.0, Operation::Div, rule_set);

    let receipts: Vec<Receipt> = vec![receipt1]; //, receipt2, receipt3, receipt4];//, receipt3, receipt4];
    println!("Receipt vector created");
    let composite_receipt = perform_composite_prove(receipts, VERIFIABLE_PROCESSING_ID)
        .expect("Failed to prove composite receipt");
    // TODO: Implement code for retrieving receipt journal here.

    // The receipt was verified at the end of proving, but the below code is an
    // example of how someone else could verify this receipt.
    println!("Composite receipt created");
    composite_receipt.verify(COMPOSITE_PROVING_ID).unwrap();

    //println!("{:?}", composite_receipt);
}

#[tokio::main]
async fn main2() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let vpssi: VerifiableProcessingServiceServerImplementation =
        VerifiableProcessingServiceServerImplementation::default();

    println!("VerifiableProcessingService listening on {}", addr);

    Server::builder()
        .add_service(VerifiableProcessingServiceServer::new(vpssi))
        .serve(addr)
        .await?;

    Ok(())
}
