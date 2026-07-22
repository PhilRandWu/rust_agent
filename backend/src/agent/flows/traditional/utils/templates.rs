pub fn class_name_utils_code() -> String {
    r#"import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_cn_utility_code() {
        let code = class_name_utils_code();

        assert!(code.contains("clsx"));
        assert!(code.contains("tailwind-merge"));
        assert!(code.contains("export function cn"));
    }
}
