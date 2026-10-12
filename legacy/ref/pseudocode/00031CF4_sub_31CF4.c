/* ea:   0x00031CF4 */
/* name: sub_31CF4 */
/* size: 228 */
int __fastcall sub_31CF4(int a1, int a2)
{
  int v4; // r0
  int v5; // r2
  unsigned int v6; // r1
  unsigned int v7; // r5
  unsigned int v8; // r0
  unsigned int v9; // r1
  unsigned int v10; // r0
  int result; // r0
  unsigned int *v12; // r6
  int v13; // t1
  unsigned int v14; // r2
  _DWORD *v15; // r3
  unsigned int v16; // r4
  unsigned int v17; // r0
  unsigned int v18; // r2
  int v19; // r1
  unsigned int v20; // r0
  unsigned int v21; // r2
  unsigned int v22; // r1
  int v23; // r2
  int v24; // r3

  v4 = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
  v5 = *(_DWORD *)(a1 + 16);
  v6 = v4;
  v7 = *(_DWORD *)(v5 + 8);
  v8 = v7 + v4;
  if ( *(_DWORD *)(a2 + 28) != 0 )
  {
    v15 = (_DWORD *)(a1 + 144);
    v16 = v8 >> 1;
    v17 = *(_DWORD *)(a1 + 144);
    v18 = 0;
    if ( v6 >= v17 )
      *v15 = 0;
    else
      *v15 = v17 - v6;
    v19 = *(_DWORD *)(a1 + 16);
    v20 = *(_DWORD *)(v19 + 8);
    if ( v7 <= v20 )
      v18 = v20 - v7;
    *(_DWORD *)(v19 + 8) = v18;
    v21 = *(_DWORD *)(a1 + 148);
    v22 = *v15 + v16;
    *v15 = v22;
    if ( v22 > v21 )
      *v15 = v21;
    v23 = *(_DWORD *)(a1 + 16);
    v24 = 999999;
    result = *(_DWORD *)(v23 + 8);
    if ( result + v16 <= 0xF423F )
      v24 = result + v16;
    *(_DWORD *)(v23 + 8) = v24;
  }
  else
  {
    if ( v8 > 0xF423F )
      v7 = 999999 - v6;
    v9 = *(_DWORD *)(v5 + 8);
    v10 = 0;
    if ( v7 <= v9 )
      v10 = v9 - v7;
    *(_DWORD *)(v5 + 8) = v10;
    v13 = *(_DWORD *)(a1 + 144);
    v12 = (unsigned int *)(a1 + 144);
    result = v13;
    v14 = v12[1];
    *v12 = v7 + v13;
    if ( v7 + v13 > v14 )
      *v12 = v14;
  }
  return result;
}

