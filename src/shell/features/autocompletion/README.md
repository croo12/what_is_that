# Autocompletion Module

이 모듈은 쉘의 자동완성 기능을 제공합니다. 사용자의 입력을 기반으로 내장 명령어, 명령어 히스토리, 파일 시스템 경로 등 다양한 소스로부터 제안 목록을 생성합니다.

## 아키텍처

자동완성 기능은 여러 개의 독립적인 "Provider"와 이를 통합 관리하는 `Autocompleter`로 구성됩니다.

### `mod.rs`

이 파일은 `autocompletion` 모듈의 진입점 역할을 합니다.

-   **Provider 모듈 선언:** `builtin_provider`, `history_provider`, `path_provider`를 하위 모듈로 선언합니다.
-   **`Autocompleter` 구조체:**
    -   자동완성 기능의 핵심 구조체입니다.
    -   `CommandHistory`와 같은 외부 의존성을 가집니다.
-   **제안 통합:**
    -   `get_suggestions` 메소드는 모든 Provider에게 비동기적으로(`tokio::join!`) 제안을 요청합니다.
    -   각 Provider로부터 받은 제안 목록을 하나로 합치고, 중복을 제거한 뒤 최종 결과를 반환합니다.

### Providers

각 Provider는 특정 종류의 자동완성 제안을 생성하는 책임을 가집니다.

-   **`builtin_provider.rs`**:
    -   `ls`, `cd`, `mkdir`, **`alias`, `unalias`** 등 쉘에 내장된 명령어 목록을 기반으로 제안을 생성합니다.
    -   사용자가 명령어의 첫 부분을 입력하고 있을 때 활성화됩니다.

-   **`history_provider.rs`**:
    -   `CommandHistory`에 저장된 이전 명령어들을 기반으로 제안을 생성합니다.
    -   입력값이 비어있을 때는 최근 사용한 명령어를, 입력값이 있을 때는 해당 입력으로 시작하는 명령어를 제안합니다.

-   **`path_provider.rs`**:
    -   현재 작업 디렉토리(`current_dir`)를 기준으로 파일 및 디렉토리 경로를 제안합니다.
    -   `shlex`를 사용하여 입력된 명령어를 파싱하고, 마지막 인자를 기반으로 경로를 탐색하여 제안을 생성합니다.
    -   공백이 포함된 경로명이나 `/`로 끝나는 디렉토리 경로 등 복잡한 케이스를 처리합니다.

이러한 분리된 구조 덕분에, 향후 새로운 자동완성 소스(예: Git 브랜치, 환경 변수)가 필요할 경우 새로운 Provider 파일만 추가하면 되므로 확장이 매우 용이합니다.