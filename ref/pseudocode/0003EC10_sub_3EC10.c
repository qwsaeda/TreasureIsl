/* ea:   0x0003EC10 */
/* name: sub_3EC10 */
/* size: 52 */
_WORD *__fastcall sub_3EC10(int a1, int a2)
{
  _WORD *v3; // r3

  v3 = sub_3E15C(a1, 1);
  if ( v3 != (_WORD *)-16 )
  {
    v3[8] = *(_WORD *)a2;
    *((_DWORD *)v3 + 5) = *(_DWORD *)(a2 + 4);
  }
  return v3;
}

