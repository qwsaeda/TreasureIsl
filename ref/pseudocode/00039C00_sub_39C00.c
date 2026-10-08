/* ea:   0x00039C00 */
/* name: sub_39C00 */
/* size: 52 */
_DWORD *__fastcall sub_39C00(int a1, int a2)
{
  _DWORD *v3; // r3

  v3 = sub_34DF0(a1, 1);
  if ( v3 != (_DWORD *)-16 )
  {
    v3[4] = *(_DWORD *)a2;
    *((_WORD *)v3 + 10) = *(_WORD *)(a2 + 4);
  }
  return v3;
}

