# Escape error narrowing (next suspects)

## Current status
- `parse_decl`의 `ret` 분기를 블록 반환 없이 `mut` 변수로 정리했지만 `value escapes its defining block` 에러는 지속됩니다.

## Next suspects (priority)
1) `parse_block_expr` (`compiler/parser.gaut:863` 부근)
   - 블록 분기에서 `{ ... }` 레코드 리턴 패턴이 많아 escape 판단 후보.
2) `parse_if_expr` (`compiler/parser.gaut:813` 부근)
   - if/else 분기마다 레코드 리턴이 반복됨.
3) `parse_record_expr` / `parse_field_inits` (`compiler/parser.gaut:882` ~ `893`)
   - 레코드 리턴 + 분기 조합.
4) `parse_primary` (`compiler/parser.gaut:924` 부근)
   - 여러 분기에서 레코드 리턴.

## Proposed action
- 위 순서대로 블록 리턴을 `mut` 변수 + 분기 내부 할당 방식으로 바꿔 escape 발생 지점을 제거합니다.

## Confirmation
- 위 순서대로 1) `parse_block_expr` 부터 수정 진행할까요?
