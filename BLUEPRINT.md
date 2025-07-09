# 리팩토링 계획: UI와 핵심 로직의 완전한 분리

## 1. 문제점

현재 `src` 디렉토리 바로 아래에 `shell_core`, `features`, `gui`, `command_history`가 함께 위치하여, 애플리케이션의 두 축인 '셸 로직'과 'GUI'의 경계가 모호하다.

## 2. 개선 목표

프로젝트 전체 아키텍처를 "라이브러리 + 실행 파일" 패턴과 유사하게 개선한다. 셸의 모든 핵심 로직을 하나의 큰 `shell` 모듈로 묶고, `gui`는 이 모듈을 사용하는 소비자로 만들어 책임과 역할을 명확히 분리한다.

## 3. 리팩토링 계획

### 변경 전 구조:

```
src/
├── shell_core/
├── features/
├── gui/
├── command_history/
└── main.rs
```

### 변경 후 구조:

```
src/
├── shell/
│   ├── mod.rs      (셸 라이브러리의 루트)
│   ├── core/       (기존 shell_core)
│   ├── features/   (부가 기능)
│   └── history/    (기존 command_history)
├── gui/
│   └── ...
└── main.rs         (두 모듈을 연결하여 애플리케이션 실행)
```

## 4. 세부 실행 단계

1.  **디렉토리 생�� 및 이동:**
    -   `src/shell` 디렉토리를 생성한다.
    -   `src/shell_core/`를 `src/shell/core/`로 이동 및 개명한다.
    -   `src/features/`를 `src/shell/features/`로 이동한다.
    -   `src/command_history/`를 `src/shell/history/`로 이동 및 개명한다.

2.  **모듈 선언 수정:**
    -   `src/main.rs`에 `pub mod shell;`을 추가하고, 기존 모듈 선언(`shell_core`, `features`, `command_history`)을 제거한다.
    -   `src/shell/mod.rs` 파일을 생성하여 `core`, `features`, `history`를 하위 모듈로 선언한다.

3.  **`use` 경로 전면 수정:**
    -   프로젝트 전체에서 `crate::shell_core`, `crate::features`, `crate::command_history`를 사용하던 모든 경로를 `crate::shell::core`, `crate::shell::features`, `crate::shell::history` 등으로 수정한다.

4.  **검증:**
    -   `cargo check`와 `cargo test`를 통해 모든 변경사항이 올바르게 적용되었는지 확인한다.