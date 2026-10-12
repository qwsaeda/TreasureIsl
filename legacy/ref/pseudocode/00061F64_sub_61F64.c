/* ea:   0x00061F64 */
/* name: sub_61F64 */
/* size: 52 */
_DWORD *__fastcall sub_61F64(_DWORD *a1, _DWORD *a2)
{
  *a1 = *a2;
  sub_132FC(a1 + 1, a2 + 1);
  sub_603EC(a1 + 4, (int)(a2 + 4));
  return a1;
}

