# 기능 청사진: Git 연동

이 문서는 쉘에 Git 연동 기능을 추가하기 위한 개발 계획을 정의합니다. 1차 목표는 쉘 프롬프트에 현재 Git 브랜치명과 저장소의 상태를 표시하는 것입니다.

## 1. 목표

- 사용자가 Git 저장소 안에 있을 때, 프롬프트에 `(main *)`과 같은 형태로 현재 브랜치와 변경 상태를 표시한다.
  - `main`: 현재 브랜치 이름
  - `*`: Staging되지 않은 변경 사항이 있음을 의미

## 2. 개발 단계

### 1단계: 의존성 추가

- **`Cargo.toml`**: Rust에서 Git 저장소를 프로그래매틱하게 다루기 위해 `git2` 라이브러리를 의존성에 추가합니다.

### 2단계: Git 정보 조회 모듈 구현

- **`src/shell_core/git.rs`** 파일을 새로 생성합니다.
- 이 모듈은 다음 기능을 책임집니다.
  - **`GitInfo` 구조체 정의:** 브랜치 이름(`String`), 변경 상태(`bool`) 등의 정보를 담는 구조체를 정의합니다.
  - **저장소 탐색:** 주어진 경로(`Path`)에서 시작하여 상위 디렉토리로 이동하며 `.git` 디렉토리를 찾아 `git2::Repository`를 엽니다.
  - **브랜치 이름 조회:** `HEAD`가 가리키는 브랜치의 이름을 가져옵니다.
  - **저장소 상태 조회:** `git2::Repository::statuses()`를 사용하여 Staging되지 않은 변경(modified, new, deleted 등)이 있는지 여부를 확인합니다.
  - 위 기능들을 종합하여 `GitInfo` 인스턴스를 반환하는 public 함수를 제공합니다.

### 3단계: `ShellCore`와 연동

- **`src/shell_core/mod.rs`**:
  - `ShellCore` 구조체에 `git_info: Option<GitInfo>` 필드를 추가하여 현재 Git 상태를 캐싱합니다.
  - `ShellCore`에 `update_git_info(&mut self)` 메소드를 추가합니다. 이 메소드는 현재 디렉토리를 기준으로 `git.rs` 모듈을 호출하여 `git_info` 필드를 갱신합니다.
- **`command_executor.rs`**:
  - `cd`와 같이 현재 디렉토리를 변경하는 명령어가 성공적으로 실행된 후, `update_git_info`를 호출하여 Git 상태를 최신으로 유지합니다.
- **`ShellCore::new()`**:
  - 쉘이 처음 시작될 때도 `update_git_info`를 호출하여 초기 Git 상태를 설정합니다.

### 4단계: GUI 프롬프트에 표시

- **`src/gui/app.rs`**:
  - 프롬프트를 렌더링하는 부분에서 `ShellCore`의 `git_info` 필드를 확인합니다.
  - `git_info`가 `Some`이면, 이를 `(main *)`과 같은 형식의 문자열로 포맷팅하여 프롬프트 경로 뒤에 추가합니다.
