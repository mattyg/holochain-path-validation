use hdi::prelude::*;

// Utility Validations
fn base_is_root_hash(action: CreateLink, prefix_index: PrefixIndex) -> ExternResult<ValidateCallbackResult> {
  if action.base_address != root_hash()? {
      return Ok(ValidateCallbackResult::Invalid("CreateLink base address must be root_hash".into()));
  }

  Ok(ValidateCallbackResult::Valid)
}

fn target_is_tag(action: CreateLink, prefix_index: PrefixIndex) -> ExternResult<ValidateCallbackResult> {
  let tag_bytes = SerializedBytes::try_from(UnsafeBytes::from(action.tag.into_inner()))
      .map_err(|_| wasm_error!("Failed to convert link tag to SerializedBytes"))?;
  let tag_component = Component::try_from(tag_bytes).map_err(|e| wasm_error!(e))?;
  let tag_string = String::try_from(&tag_component).map_err(|e| wasm_error!(e))?;

  if EntryHash::from(action.target_address) != Path::from(tag_string).path_entry_hash()? {
      return Ok(ValidateCallbackResult::Invalid("CreateLink target must be tag as component".into()));
  }

  Ok(ValidateCallbackResult::Valid)
}

fn tag_is_expected_string(action: CreateLink,  prefix_index: PrefixIndex) -> ExternResult<ValidateCallbackResult> {
  let tag_bytes = SerializedBytes::try_from(UnsafeBytes::from(action.tag.into_inner()))
      .map_err(|_| wasm_error!("Failed to convert link tag to SerializedBytes"))?;
  let tag_component = Component::try_from(tag_bytes).map_err(|e| wasm_error!(e))?;
  let tag_string = String::try_from(&tag_component).map_err(|e| wasm_error!(e))?;

  if let Some(expected) = payload {
      let expected_string: String = holochain_serialized_bytes::decode(&expected).unwrap();

      if tag_string == expected_string {
          return Ok(ValidateCallbackResult::Valid);
      }
  }

  Ok(ValidateCallbackResult::Invalid("CreateLink tag must be Component matching expected string".into()))
}

fn tag_has_expected_chars_count(action: CreateLink,  prefix_index: PrefixIndex) -> ExternResult<ValidateCallbackResult> {
  let tag_bytes = SerializedBytes::try_from(UnsafeBytes::from(action.tag.into_inner()))
      .map_err(|_| wasm_error!("Failed to convert link tag to SerializedBytes"))?;
  let tag_component = Component::try_from(tag_bytes).map_err(|e| wasm_error!(e))?;
  let tag_string = String::try_from(&tag_component).map_err(|e| wasm_error!(e))?;

  if let Some(expected) = payload {
      let expected_chars_count: usize = holochain_serialized_bytes::decode(&expected).unwrap();

      if tag_string.chars().count() == expected_chars_count {
          return Ok(ValidateCallbackResult::Valid);
      }
  }

  return Ok(ValidateCallbackResult::Invalid("CreateLink tag must have expected number of characters".into()));
}

fn always_invalid(reason: String) -> ExternResult<ValidateCallbackResult> {
  Ok(ValidateCallbackResult::Invalid(reason))
}

fn base_is_prev_action_target(action: CreateLink,  prefix_index: PrefixIndex) -> ExternResult<ValidateCallbackResult> {
  let prev_action = must_get_action(action.prev_action)?;
      
  if let Action::CreateLink(prev_create_link) = prev_action.hashed.as_content() {
      if action.base_address == prev_create_link.target_address {
          return Ok(ValidateCallbackResult::Valid)
      }
  }

  Ok(ValidateCallbackResult::Invalid("CreateLink base must be previous action's target".into()))
}

fn tag_is_superstring_of_expected_prev_actions_tags(action: CreateLink, prefix_index: PrefixIndex) -> ExternResult<ValidateCallbackResult> {
  if let Some(expected) = payload {
      let expected_prev_actions: u32 = holochain_serialized_bytes::decode(&expected).unwrap();

      let prev_actions = must_get_agent_activity(action.author, ChainFilter::new(action.prev_action).take(expected_prev_actions-1))?;
      
      let prev_action_tags: Vec<CreateLink> = prev_actions
          .iter()
          .map(|a| -> ExternResult<CreateLink> {
              match a.action.hashed.content.clone() {
                  Action::CreateLink(create_link) => Ok(create_link),
                  _ => Err(wasm_error!(WasmErrorInner::Guest("Expected CreateLink Action".into())))
              }
          })
          .collect::<ExternResult<Vec<CreateLink>>>()?;

      let prev_action_tag_strings: Vec<String> = prev_action_tags
          .iter()
          .map(|t| {
              let tag_bytes = SerializedBytes::try_from(UnsafeBytes::from(t.tag.clone().into_inner()))
                  .map_err(|_| wasm_error!("Failed to convert link tag to SerializedBytes"))?;
              let tag_component = Component::try_from(tag_bytes).map_err(|e| wasm_error!(e))?;
              let tag_string = String::try_from(&tag_component).map_err(|e| wasm_error!(e))?;
              
              Ok(tag_string)
          })
          .collect::<ExternResult<Vec<String>>>()?;
      
      let tag_bytes = SerializedBytes::try_from(UnsafeBytes::from(action.tag.into_inner()))
          .map_err(|_| wasm_error!("Failed to convert link tag to SerializedBytes"))?;
      let tag_component = Component::try_from(tag_bytes).map_err(|e| wasm_error!(e))?;
      let tag_string = String::try_from(&tag_component).map_err(|e| wasm_error!(e))?;

      if !tag_string.contains(&prev_action_tag_strings.join("").to_string()) {
          return Ok(ValidateCallbackResult::Invalid(format!("Previous {:?} actions must be CreateLink with tags that form a substring of current action tag '{:?}'", expected, tag_string)))
      }

  }
  Ok(ValidateCallbackResult::Valid)
}
