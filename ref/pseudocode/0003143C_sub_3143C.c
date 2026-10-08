/* ea:   0x0003143C */
/* name: sub_3143C */
/* size: 100 */
int __fastcall sub_3143C(int a1, int a2, unsigned __int16 a3)
{
  int v5; // r4
  int v6; // r3
  int v7; // r0
  int v8; // r4
  int v9; // r3
  int v10; // r2

  v5 = *(_DWORD *)(a1 + 16);
  v6 = 999999;
  v7 = *(_DWORD *)(v5 + 8);
  if ( (unsigned int)(v7 + a2) <= 0xF423F )
    v6 = v7 + a2;
  *(_DWORD *)(v5 + 8) = v6;
  v8 = *(_DWORD *)(a1 + 16);
  v9 = 999;
  v10 = *(unsigned __int16 *)(v8 + 12);
  if ( v10 + (unsigned int)a3 <= 0x3E7 )
    v9 = v10 + a3;
  *(_WORD *)(v8 + 12) = v9;
  return v9 << 16;
}

