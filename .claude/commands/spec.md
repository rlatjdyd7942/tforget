$ARGUMENTS

--

1. 먼저 librarian 서브에이전트를 사용하여 위 요구사항과 관련된 문서와 코드를 파악해. figma 링크는 figma-code-reader 서브에이전트를 사용하여 필요한 코드를 파악해.

2. librarian 결과를 바탕으로 스펙 문서를 수정해:
   - SPEC.md: 전체 프로젝트 스펙 인덱스
   - docs/specs/core/: 데이터 모델
   - docs/specs/admin/: 어드민 API, 인증, 매핑 대기열, UI 가이드
   - docs/specs/frontend/pages/: 페이지별 스펙문서
   - 백엔드 전용: backend/SPEC.md, backend/specs/

3. 요구사항에 따라 적절한 스펙 파일을 수정하고, PLAN.md에 구현 계획을 추가하고 참조해야할 스펙문서 링크해. 스펙문서에 수정하려고 하는 코드의 문제점 등을 쓰지마. 스펙은 "어떻게 되어야 하는지"를 쓰는 곳이지 문제점을 기록하는 곳이 아니야.
