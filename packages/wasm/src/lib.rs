use std::array::IntoIter;
use wasm_bindgen::prelude::*;

use parity_scale_codec::{Decode, Encode};
use smoldot::{
    executor::{
        host::{Config, HeapPages, HostVmPrototype, StorageProofSizeBehavior},
        runtime_call::{self, RuntimeCall},
        storage_diff::TrieDiff,
    },
    json_rpc::methods::HexString,
};

// Constants for metadata version preferences
const PREFERRED_VERSIONS: &[u32] = &[16, 15, 14];

/// Creates a configured VM prototype from a WASM blob
pub(crate) fn create_vm_prototype(wasm_blob: HexString) -> Result<HostVmPrototype, JsValue> {
    HostVmPrototype::new(Config {
        module: wasm_blob,
        heap_pages: HeapPages::from(2048),
        exec_hint: smoldot::executor::vm::ExecHint::ValidateAndExecuteOnce,
        allow_unresolved_imports: true,
    })
    .map_err(|e| serde_wasm_bindgen::to_value(&format!("Failed to create VM: {e:?}"))
        .unwrap_or_else(|_| JsValue::from_str("Failed to create VM")))
}

/// Detects the best available metadata version from the runtime
fn detect_metadata_version(vm_proto: &HostVmPrototype) -> Option<u32> {
    let result = execute_runtime_call(
        vm_proto.clone(),
        "Metadata_metadata_versions",
        vec![]
    );

    match result {
        Ok(hex_data) => {
            Vec::<u32>::decode(&mut &hex_data.0[..])
                .ok()
                .and_then(|available_versions| {
                    PREFERRED_VERSIONS
                        .iter()
                        .find(|&&v| available_versions.contains(&v))
                        .copied()
                })
        }
        Err(_) => None,
    }
}

/// Fetches metadata for a specific version
fn fetch_metadata_at_version(
    vm_proto: &HostVmPrototype,
    version: u32,
) -> Result<HexString, JsValue> {
    let version_param = HexString(version.encode());
    let result = execute_runtime_call(
        vm_proto.clone(),
        "Metadata_metadata_at_version",
        vec![version_param]
    )?;

    // metadata_at_version returns an Option<OpaqueMetadata>
    // The first byte indicates Some(1) or None(0)
    // We verified the version exists, so we skip the first byte
    if result.0.is_empty() {
        return Err(JsValue::from_str("Empty metadata response"));
    }

    // Decode the Vec<u8> to unwrap the compact length prefix
    let metadata_bytes = Vec::<u8>::decode(&mut &result.0[1..])
        .map_err(|_| JsValue::from_str("Failed to decode metadata bytes"))?;

    Ok(HexString(metadata_bytes))
}

/// Fetches metadata using the legacy method (no versioning)
pub(crate) fn fetch_metadata_legacy(vm_proto: &HostVmPrototype) -> Result<HexString, JsValue> {
    let result = execute_runtime_call(vm_proto.clone(), "Metadata_metadata", vec![])?;

    // Decode the Vec<u8> to unwrap the compact length prefix
    let metadata_bytes = Vec::<u8>::decode(&mut &result.0[..])
        .map_err(|_| JsValue::from_str("Failed to decode legacy metadata bytes"))?;

    Ok(HexString(metadata_bytes))
}

/// Executes a runtime call with the given function and parameters
fn execute_runtime_call(
    vm_proto: HostVmPrototype,
    function_name: &str,
    parameters: Vec<HexString>,
) -> Result<HexString, JsValue> {
    let vm_result = runtime_call::run(runtime_call::Config {
        virtual_machine: vm_proto,
        function_to_call: function_name,
        parameter: parameters.into_iter().map(|hex| hex.0),
        storage_main_trie_changes: TrieDiff::default(),
        max_log_level: 0,
        calculate_trie_changes: false,
        storage_proof_size_behavior: StorageProofSizeBehavior::ConstantReturnValue(0),
    });

    match vm_result {
        Ok(runtime_call) => extract_call_result(runtime_call),
        Err(_) => Err(serde_wasm_bindgen::to_value(
            &format!("Runtime call '{function_name}' failed")
        )?),
    }
}

/// Recursively extracts the result from a runtime call
fn extract_call_result(call: RuntimeCall) -> Result<HexString, JsValue> {
    match call {
        RuntimeCall::Finished(outcome) => match outcome {
            Ok(success) => {
                let value = success.virtual_machine.value().as_ref().to_vec();
                Ok(HexString(value))
            }
            Err(_) => Err(JsValue::from_str("Runtime call execution failed")),
        },
        RuntimeCall::StorageGet(storage_request) => {
            // Inject None for storage requests (no storage proofs needed)
            let next_call = storage_request.inject_value(Option::<(IntoIter<Vec<u8>, 0>, _)>::None);
            extract_call_result(next_call)
        }
        _ => Err(JsValue::from_str("Unexpected VM state during execution")),
    }
}

/// Internal implementation that extracts metadata from a WASM runtime blob.
///
/// This function is separated from the wasm_bindgen wrapper to enable native testing.
/// It automatically detects the best available metadata version (preferring v16, then v15, then v14)
/// and falls back to legacy metadata extraction if version detection is not supported.
fn get_metadata(wasm: HexString) -> Result<HexString, String> {
    let vm_prototype = create_vm_prototype(wasm)
        .map_err(|e| format!("Failed to create VM prototype: {e:?}"))?;

    // Try to detect and use versioned metadata
    let metadata = match detect_metadata_version(&vm_prototype) {
        Some(version) => fetch_metadata_at_version(&vm_prototype, version)
            .map_err(|e| format!("Failed to fetch metadata at version {version}: {e:?}"))?,
        None => fetch_metadata_legacy(&vm_prototype)
            .map_err(|e| format!("Failed to fetch legacy metadata: {e:?}"))?,
    };

    Ok(metadata)
}

// TypeScript type definitions
#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
type HexString = `0x${string}`

/**
 * Get metadata from WASM runtime.
 *
 * @param {HexString} wasmBlob - The runtime WASM blob as a hex string
 * @returns {HexString} The runtime metadata as a hex string
 */
export function getMetadataFromWasmRuntime(wasmBlob: HexString): HexString"#;

/// Get metadata from WASM runtime.
///
/// This function extracts metadata from a Substrate/Polkadot runtime WASM blob.
/// It automatically detects the best available metadata version (preferring v16, then v15, then v14)
/// and falls back to legacy metadata extraction if version detection is not supported.
///
/// @param {HexString} wasmBlob - The runtime WASM blob as a hex string
/// @returns {HexString} The runtime metadata as a hex string
#[wasm_bindgen(js_name = getMetadataFromWasmRuntime, skip_typescript)]
pub fn get_metadata_from_wasm_runtime(wasm_blob: JsValue) -> Result<JsValue, JsValue> {
    let wasm = serde_wasm_bindgen::from_value::<HexString>(wasm_blob)?;

    let metadata = get_metadata(wasm)
        .map_err(|e| JsValue::from_str(&e))?;

    Ok(serde_wasm_bindgen::to_value(&metadata)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
    use parity_scale_codec::Decode;
    use std::fs;

    /// Helper function to load runtime.wasm and convert to HexString
    fn load_runtime_wasm() -> HexString {
        let wasm_bytes = fs::read("./tests_data/runtime.wasm")
            .expect("Failed to read runtime.wasm file");

        // HexString wraps raw bytes, not hex-encoded strings
        HexString(wasm_bytes)
    }

    #[test]
    fn test_get_metadata_from_wasm_runtime() {
        // Load the runtime WASM blob
        let wasm_blob = load_runtime_wasm();

        // Extract metadata from the runtime
        let metadata_hex = get_metadata(wasm_blob)
            .expect("Failed to extract metadata from runtime");

        // Decode the metadata using RuntimeMetadataPrefixed
        // The length prefix has already been unwrapped by fetch_metadata_at_version
        let metadata: RuntimeMetadataPrefixed = Decode::decode(&mut metadata_hex.0.as_slice())
            .expect("Failed to decode RuntimeMetadataPrefixed");

        // Verify it's V15 metadata as expected
        assert!(
            matches!(metadata.1, RuntimeMetadata::V15(_)),
            "Expected V15 metadata, got {:?}",
            metadata.1
        );

        println!("Successfully decoded metadata V15 with {} pallets",
            if let RuntimeMetadata::V15(ref v15) = metadata.1 {
                v15.pallets.len()
            } else {
                0
            }
        );
    }

    #[test]
    fn test_fetch_metadata_legacy() {
        // Load the runtime WASM blob
        let wasm_blob = load_runtime_wasm();

        // Create VM prototype
        let vm_prototype = create_vm_prototype(wasm_blob)
            .expect("Failed to create VM prototype");

        // Test the legacy metadata extraction path directly
        // Even though this runtime supports versioned metadata, the legacy
        // Metadata_metadata function should still work for backward compatibility
        let metadata_hex = fetch_metadata_legacy(&vm_prototype)
            .expect("Failed to fetch legacy metadata");

        // Decode the metadata using RuntimeMetadataPrefixed
        let metadata: RuntimeMetadataPrefixed = Decode::decode(&mut metadata_hex.0.as_slice())
            .expect("Failed to decode RuntimeMetadataPrefixed from legacy metadata");

        // Verify the metadata is valid (version can be any, we're testing the legacy path works)
        match metadata.1 {
            RuntimeMetadata::V14(_) => println!("Legacy metadata: V14"),
            RuntimeMetadata::V15(_) => println!("Legacy metadata: V15"),
            RuntimeMetadata::V16(_) => println!("Legacy metadata: V16"),
            _ => panic!("Unexpected metadata version: {:?}", metadata.1),
        }

        println!("Successfully decoded legacy metadata with magic: {:?}",
            &metadata.0
        );
    }
}
