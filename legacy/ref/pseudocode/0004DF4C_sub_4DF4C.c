/* ea:   0x0004DF4C */
/* name: sub_4DF4C */
/* size: 72 */
int __fastcall sub_4DF4C(int a1, unsigned __int8 a2)
{
  _BYTE *v3; // r3
  int result; // r0

  v3 = sub_19DE4(*(_BYTE **)a1, *(_DWORD *)(a1 + 4), a2);
  result = 0;
  if ( v3 != *(_BYTE **)(a1 + 4) )
    return *((_DWORD *)v3 + 1);
  return result;
}

