/* ea:   0x00016D1C */
/* name: sub_16D1C */
/* size: 92 */
int __fastcall sub_16D1C(int a1)
{
  _DWORD *v2; // r2
  int v3; // r5

  v2 = *(_DWORD **)(*(_DWORD *)a1 + 8);
  v3 = 0;
  while ( v2 != *(_DWORD **)a1 )
  {
    v3 += *((unsigned __int16 *)v2 + 9);
    v2 = sub_16A30(v2);
  }
  return v3;
}

