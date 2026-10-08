/* ea:   0x0005467C */
/* name: sub_5467C */
/* size: 100 */
_DWORD *__fastcall sub_5467C(int a1, _DWORD *a2, unsigned __int8 a3)
{
  if ( a3 != 0 )
    *a2 = (*(int *)(a1 + 20) * (__int64)*(int *)(*(_DWORD *)(a1 + 24) + 4 * a3 - 4)) >> 16;
  else
    *a2 = *(_DWORD *)(a1 + 20);
  return a2;
}

