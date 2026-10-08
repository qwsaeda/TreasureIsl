/* ea:   0x0003402C */
/* name: sub_3402C */
/* size: 56 */
_DWORD *__fastcall sub_3402C(int a1)
{
  *(_DWORD *)(a1 + 48) = 0;
  *(_DWORD *)(a1 + 52) = 0;
  sub_305A8(a1 + 28, *(_DWORD **)(a1 + 28), *(_DWORD *)(a1 + 32));
  return sub_305A8(a1 + 16, *(_DWORD **)(a1 + 16), *(_DWORD *)(a1 + 20));
}

