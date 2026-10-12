/* ea:   0x0001980C */
/* name: sub_1980C */
/* size: 64 */
char *__fastcall sub_1980C(int a1, char *a2, _BYTE *a3)
{
  _BYTE *v5; // r4
  char *v6; // r0

  v5 = *(_BYTE **)(a1 + 4);
  v6 = a2;
  if ( v5 != a3 )
    v6 = &v5[(_BYTE *)memmove(a2, a3, v5 - a3) - a3];
  *(_DWORD *)(a1 + 4) = v6;
  return a2;
}

