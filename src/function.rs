#![allow(unknown_lints)]
#![allow(clippy::unnecessary_wraps)]

use super::*;

use heck::{
  ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase,
  ToUpperCamelCase,
};

use Function::*;

pub(crate) enum Function {
  Nullary(fn(&FunctionContext) -> Result<String, String>),
  Unary(fn(&FunctionContext, &str) -> Result<String, String>),
  Binary(fn(&FunctionContext, &str, &str) -> Result<String, String>),
  BinaryPlus(fn(&FunctionContext, &str, &str, &[String]) -> Result<String, String>),
  Ternary(fn(&FunctionContext, &str, &str, &str) -> Result<String, String>),
}

pub(crate) fn get(name: &str) -> Option<Function> {
  let function = match name {
    "absolute_path" => Unary(absolute_path),
    "arch" => Nullary(arch),
    "capitalize" => Unary(capitalize),
    "clean" => Unary(clean),
    "env_var" => Unary(env_var),
    "env_var_or_default" => Binary(env_var_or_default),
    "error" => Unary(error),
    "extension" => Unary(extension),
    "file_name" => Unary(file_name),
    "file_stem" => Unary(file_stem),
    "invocation_directory" => Nullary(invocation_directory),
    "join" => BinaryPlus(join),
    "just_executable" => Nullary(just_executable),
    "justfile" => Nullary(justfile),
    "justfile_directory" => Nullary(justfile_directory),
    "kebabcase" => Unary(kebabcase),
    "lowercamelcase" => Unary(lowercamelcase),
    "lowercase" => Unary(lowercase),
    "os" => Nullary(os),
    "os_family" => Nullary(os_family),
    "parent_directory" => Unary(parent_directory),
    "path_exists" => Unary(path_exists),
    "quote" => Unary(quote),
    "replace" => Ternary(replace),
    "replace_regex" => Ternary(replace_regex),
    "sha256" => Unary(sha256),
    "sha256_file" => Unary(sha256_file),
    "shoutykebabcase" => Unary(shoutykebabcase),
    "shoutysnakecase" => Unary(shoutysnakecase),
    "snakecase" => Unary(snakecase),
    "titlecase" => Unary(titlecase),
    "trim" => Unary(trim),
    "trim_end" => Unary(trim_end),
    "trim_end_match" => Binary(trim_end_match),
    "trim_end_matches" => Binary(trim_end_matches),
    "trim_start" => Unary(trim_start),
    "trim_start_match" => Binary(trim_start_match),
    "trim_start_matches" => Binary(trim_start_matches),
    "uppercamelcase" => Unary(uppercamelcase),
    "uppercase" => Unary(uppercase),
    "uuid" => Nullary(uuid),
    "without_extension" => Unary(without_extension),
    _ => return None,
  };
  Some(function)
}

impl Function {
  pub(crate) fn argc(&self) -> Range<usize> {
    match *self {
      Nullary(_) => 0..0,
      Unary(_) => 1..1,
      Binary(_) => 2..2,
      BinaryPlus(_) => 2..usize::MAX,
      Ternary(_) => 3..3,
    }
  }
}

fn absolute_path(context: &FunctionContext, path: &str) -> Result<String, String> {
  let abs_path_unchecked = context.search.working_directory.join(path).lexiclean();
  match abs_path_unchecked.to_str() {
    Some(absolute_path) => Ok(absolute_path.to_owned()),
    None => Err(format!(
      "Working directory is not valid unicode: {}",
      context.search.working_directory.display()
    )),
  }
}

fn arch(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::arch().to_owned())
}

fn capitalize(_context: &FunctionContext, s: &str) -> Result<String, String> {
  let mut capitalized = String::new();
  for (i, c) in s.chars().enumerate() {
    if i == 0 {
      capitalized.extend(c.to_uppercase());
    } else {
      capitalized.extend(c.to_lowercase());
    }
  }
  Ok(capitalized)
}

fn clean(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Ok(Path::new(path).lexiclean().to_str().unwrap().to_owned())
}

fn env_var(context: &FunctionContext, key: &str) -> Result<String, String> {
  use std::env::VarError::*;

  if let Some(value) = context.dotenv.get(key) {
    return Ok(value.clone());
  }

  match env::var(key) {
    Err(NotPresent) => Err(format!("environment variable `{key}` not present")),
    Err(NotUnicode(os_string)) => Err(format!(
      "environment variable `{}` not unicode: {:?}",
      key, os_string
    )),
    Ok(value) => Ok(value),
  }
}

fn env_var_or_default(
  context: &FunctionContext,
  key: &str,
  default: &str,
) -> Result<String, String> {
  use std::env::VarError::*;

  if let Some(value) = context.dotenv.get(key) {
    return Ok(value.clone());
  }

  match env::var(key) {
    Err(NotPresent) => Ok(default.to_owned()),
    Err(NotUnicode(os_string)) => Err(format!(
      "environment variable `{}` not unicode: {:?}",
      key, os_string
    )),
    Ok(value) => Ok(value),
  }
}

fn error(_context: &FunctionContext, message: &str) -> Result<String, String> {
  Err(message.to_owned())
}

fn extension(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Utf8Path::new(path)
    .extension()
    .map(str::to_owned)
    .ok_or_else(|| format!("Could not extract extension from `{path}`"))
}

fn file_name(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Utf8Path::new(path)
    .file_name()
    .map(str::to_owned)
    .ok_or_else(|| format!("Could not extract file name from `{path}`"))
}

fn file_stem(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Utf8Path::new(path)
    .file_stem()
    .map(str::to_owned)
    .ok_or_else(|| format!("Could not extract file stem from `{path}`"))
}

fn invocation_directory(context: &FunctionContext) -> Result<String, String> {
  Platform::convert_native_path(
    &context.search.working_directory,
    context.invocation_directory,
  )
  .map_err(|e| format!("Error getting shell path: {e}"))
}

fn join(
  _context: &FunctionContext,
  base: &str,
  with: &str,
  and: &[String],
) -> Result<String, String> {
  let mut result = Utf8Path::new(base).join(with);
  for arg in and {
    result.push(arg);
  }
  Ok(result.to_string())
}

fn just_executable(_context: &FunctionContext) -> Result<String, String> {
  let exe_path =
    std::env::current_exe().map_err(|e| format!("Error getting current executable: {e}"))?;

  exe_path.to_str().map(str::to_owned).ok_or_else(|| {
    format!(
      "Executable path is not valid unicode: {}",
      exe_path.display()
    )
  })
}

fn justfile(context: &FunctionContext) -> Result<String, String> {
  context
    .search
    .justfile
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "Justfile path is not valid unicode: {}",
        context.search.justfile.display()
      )
    })
}

fn justfile_directory(context: &FunctionContext) -> Result<String, String> {
  let justfile_directory = context.search.justfile.parent().ok_or_else(|| {
    format!(
      "Could not resolve justfile directory. Justfile `{}` had no parent.",
      context.search.justfile.display()
    )
  })?;

  justfile_directory
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "Justfile directory is not valid unicode: {}",
        justfile_directory.display()
      )
    })
}

fn kebabcase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_kebab_case())
}

fn lowercamelcase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_lower_camel_case())
}

fn lowercase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_lowercase())
}

fn os(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::os().to_owned())
}

fn os_family(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::family().to_owned())
}

fn parent_directory(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Utf8Path::new(path)
    .parent()
    .map(Utf8Path::to_string)
    .ok_or_else(|| format!("Could not extract parent directory from `{path}`"))
}

fn path_exists(context: &FunctionContext, path: &str) -> Result<String, String> {
  Ok(
    context
      .search
      .working_directory
      .join(path)
      .exists()
      .to_string(),
  )
}

fn quote(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(format!("'{}'", s.replace('\'', "'\\''")))
}

fn replace(_context: &FunctionContext, s: &str, from: &str, to: &str) -> Result<String, String> {
  Ok(s.replace(from, to))
}

fn replace_regex(
  _context: &FunctionContext,
  s: &str,
  regex: &str,
  replacement: &str,
) -> Result<String, String> {
  Ok(
    Regex::new(regex)
      .map_err(|err| err.to_string())?
      .replace_all(s, replacement)
      .to_string(),
  )
}

fn sha256(_context: &FunctionContext, s: &str) -> Result<String, String> {
  use sha2::{Digest, Sha256};
  let mut hasher = Sha256::new();
  hasher.update(s);
  let hash = hasher.finalize();
  Ok(format!("{hash:x}"))
}

fn sha256_file(context: &FunctionContext, path: &str) -> Result<String, String> {
  use sha2::{Digest, Sha256};
  let justpath = context.search.working_directory.join(path);
  let mut hasher = Sha256::new();
  let mut file = std::fs::File::open(&justpath)
    .map_err(|err| format!("Failed to open file at `{:?}`: {err}", &justpath.to_str()))?;
  std::io::copy(&mut file, &mut hasher)
    .map_err(|err| format!("Failed to read file at `{:?}`: {err}", &justpath.to_str()))?;
  let hash = hasher.finalize();
  Ok(format!("{hash:x}"))
}

fn shoutykebabcase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_shouty_kebab_case())
}

fn shoutysnakecase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_shouty_snake_case())
}

fn snakecase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_snake_case())
}

fn titlecase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_title_case())
}

fn trim(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.trim().to_owned())
}

fn trim_end(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.trim_end().to_owned())
}

fn trim_end_match(_context: &FunctionContext, s: &str, pat: &str) -> Result<String, String> {
  Ok(s.strip_suffix(pat).unwrap_or(s).to_owned())
}

fn trim_end_matches(_context: &FunctionContext, s: &str, pat: &str) -> Result<String, String> {
  Ok(s.trim_end_matches(pat).to_owned())
}

fn trim_start(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.trim_start().to_owned())
}

fn trim_start_match(_context: &FunctionContext, s: &str, pat: &str) -> Result<String, String> {
  Ok(s.strip_prefix(pat).unwrap_or(s).to_owned())
}

fn trim_start_matches(_context: &FunctionContext, s: &str, pat: &str) -> Result<String, String> {
  Ok(s.trim_start_matches(pat).to_owned())
}

fn uppercamelcase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_upper_camel_case())
}

fn uppercase(_context: &FunctionContext, s: &str) -> Result<String, String> {
  Ok(s.to_uppercase())
}

fn uuid(_context: &FunctionContext) -> Result<String, String> {
  Ok(uuid::Uuid::new_v4().to_string())
}

fn without_extension(_context: &FunctionContext, path: &str) -> Result<String, String> {
  let parent = Utf8Path::new(path)
    .parent()
    .ok_or_else(|| format!("Could not extract parent from `{path}`"))?;

  let file_stem = Utf8Path::new(path)
    .file_stem()
    .ok_or_else(|| format!("Could not extract file stem from `{path}`"))?;

  Ok(parent.join(file_stem).to_string())
}
