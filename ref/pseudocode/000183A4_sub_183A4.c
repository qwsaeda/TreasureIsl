/* ea:   0x000183A4 */
/* name: sub_183A4 */
/* size: 88 */
int __fastcall sub_183A4(_DWORD *a1, _BYTE *a2, _BYTE *a3)
{
  size_t v6; // r4
  _WORD *v7; // r3

  v6 = a3 - a2;
  sub_13850(a1, ((a3 - a2) >> 1) + 1);
  v7 = (_WORD *)*a1;
  if ( a3 != a2 )
    v7 = &a3[(_BYTE *)memmove((void *)*a1, a2, v6) - a2];
  a1[1] = v7;
  *v7 = 0;
  return 0;
}

