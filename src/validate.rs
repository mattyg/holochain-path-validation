use hdi::prelude::*;

// Prefix Index Validation rules generator
fn make_validation_rules_for_prefix_index(prefix_index: PrefixIndex) -> Vec<Vec<impl Fn(CreateLink, Option<Vec<u8>>) -> ExternResult<ValidateCallbackResult>>>
{
  let mut rules: Vec<Vec<fn(CreateLink, Option<Vec<u8>>) -> ExternResult<ValidateCallbackResult>>> = vec!(
      vec!(
          base_is_root_hash,
          target_is_tag,
          |action| tag_is_expected_string(action, "my_string")
      )
  );

  for _ in 0..prefix_index.depth {
      rules.push(
          vec!(
              base_is_prev_action_target,
              target_is_tag,
              tag_has_expected_chars_count,
          ),
      );
  }

  rules.push(
      vec!(
          base_is_prev_action_target,
          target_is_tag,
          tag_is_superstring_of_expected_prev_actions_tags
      )
  );
  rules.push(
      vec!(
          |_, _| always_invalid("PrefixIndex has too many components".into()),
      )
  );

  rules
}


/// Validate the structure of a Path
/// 
/// As every CreateLink is validated independantly
/// From the validate callback, I don't know where this particular CreateLink is located within a path
/// To deal with this, we step backwards down my source chain actions until I reach a CreateLink where the base is root_hash (the root of the Path)
/// Then, we step forwards up until the current action, validating each component along the way
/// 
/// 
/// @todo this approach may not work *at all* because we can't know that the current agent authored *all* links in the path
///   (some may have been authored by others)
///   thus we can't review the agent activity to get all the previous links.
///   Also because we can't do a must_get for a specific link (since links are actions and we don't know the author / timestmap)
///     we also can't get the previous link by simply knowing the first link.
///   So there is no way to look back at the prior links.
/// 
/// This will require the refactored source chain where all parts of a path are committed as a bundle
///   AND links being entries, not actions

pub fn validate_create_link_within_path<F>(
  action: CreateLink,
  base_address: AnyLinkableHash,
  target_address: AnyLinkableHash,
  tag: LinkTag,
  mut path_validation_rules: Vec<Vec<F>>,
  prefix_index: PrefixIndex,
) -> ExternResult<ValidateCallbackResult> 
where 
  F: Fn(CreateLink, Option<Vec<u8>>) -> ExternResult<ValidateCallbackResult>
{
  // Step backwards until we reach the CreateLink action for the Path root
  let mut create_link_actions = vec!(action.clone());
  loop {
      let prev_action = must_get_action(create_link_actions.get(0).unwrap().prev_action.clone())?;
      
      if let Action::CreateLink(prev_create_link) = prev_action.hashed.as_content() {
          if prev_create_link.base_address == root_hash()? {
              // Begin validation from root onwards
              break;
          } else {
              // Continue traversing through previous actions
              create_link_actions.insert(0, prev_create_link.clone());
          }
      }
  }

  // Step forwards, validating each CreateLink action in the Path
  while !path_validation_rules.is_empty() && !create_link_actions.is_empty() {
      let rules = path_validation_rules.remove(0);
      let create_link = create_link_actions.remove(0);

      for rule in rules.into_iter() {
          let result = rule(create_link.clone(), prefix_index)?;
          
          if result != ValidateCallbackResult::Valid {
              return Ok(result)
          }
      }
  }

  Ok(ValidateCallbackResult::Valid)
}
