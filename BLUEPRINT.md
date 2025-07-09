# 기능 청사진: Git 연동

이 문서는 쉘에 Git 연동 기능을 추가하기 위한 개발 계획을 정의합니다. 1차 목표는 쉘 프롬프트에 현재 Git 브랜치명과 저장소의 상태를 표시하는 것입니다.

## 3. 목표: 외부 명령어 실행 기능 강화

- 사용자가 `git`, `npm`, `python` 등 시스템에 설치된 모든 외부 명령어를 우리 쉘에서 직접 실행할 수 있도록 한다.

## 4. 개발 단계

### 1단계: `PATH` 환경 변수 탐색 기능 구현

- **`src/shell_core/external.rs`**:
  - `find_executable_in_path(command_name: &str) -> Option<PathBuf>` 함수를 새로 구현합니다.
  - 이 함수는 `std::env::var("PATH")`를 통해 `PATH` 환경 변수를 가져옵니다.
  - `PATH` 문자열을 각 운영체제에 맞는 구분자(`:` 또는 `;`)로 분리하여 경로 목록을 만듭니다.
  - 각 경로를 순회하며 `command_name`에 해당하는 실행 파일(Windows의 경우 `.exe`, `.cmd` 등 확장자 포함)이 존재하는지 확인합니다.
  - 실행 파일을 찾으면 해당 전체 경로(`PathBuf`)를 반환하고, 찾지 못하면 `None`을 반환합니다.

### 2단계: 외부 명령어 실행 로직 개선

- **`src/shell_core/external.rs`**:
  - 기존 `execute_external_command` 함수를 수정합니다.
  - `find_executable_in_path`를 호출하여 실행할 명령어의 전체 경로를 얻습니다.
  - 경로를 찾았다면, `std::process::Command`를 사용하여 해당 경로의 프로그램을 실행하고, 인자(arguments)를 전달합니다.
  - 경로를 찾지 못했다면, `"command not found"`와 같은 명확한 오류 메시지를 반환합니다.
