/* ea:   0x0003AB44 */
/* name: sub_3AB44 */
/* size: 968 */
int __fastcall sub_3AB44(int a1, int a2, unsigned __int16 a3, int a4, int a5)
{
  int v8; // r0
  int i; // r4
  int v10; // r0
  int v11; // r0
  int v12; // r0
  int j; // r5
  int v14; // r0
  int v15; // r0
  int v16; // r0
  int k; // r0
  int v18; // r0
  int v19; // r0
  int m; // r0
  int v21; // r0
  int v22; // r0
  int v23; // r0
  int v24; // r0
  int n; // r0
  int v26; // r0
  int v27; // r0
  int ii; // r6
  int v29; // r0
  int v30; // r0
  int v31; // r0
  int result; // r0
  int jj; // r6
  int v34; // r0
  int v35; // r0
  int v36; // [sp+4h] [bp-64h] BYREF
  wchar_t v37[48]; // [sp+8h] [bp-60h] BYREF

  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  v8 = sub_12F04(aU);
  for ( i = sub_3B0C0(dword_81184, aU, 0, v8); i != -1; i = sub_3B0C0(dword_81184, aU, 0, v11) )
  {
    v10 = sub_186F0(a4, *(_DWORD *)(a2 + 8));
    sub_3AF0C(dword_81184, i, 2, v10);
    v11 = sub_12F04(aU);
  }
  v12 = sub_12F04(aT);
  for ( j = sub_3B0C0(dword_81184, aT, 0, v12); j != -1; j = sub_3B0C0(dword_81184, aT, 0, v15) )
  {
    v14 = sub_186F0(a4, a3);
    sub_3AF0C(dword_81184, j, 2, v14);
    v15 = sub_12F04(aT);
  }
  itow(a5, v37, 10);
  v16 = sub_12F04(aN);
  for ( k = sub_3B0C0(dword_81184, aN, 0, v16); k != -1; k = sub_3B0C0(dword_81184, aN, 0, v18) )
  {
    sub_3AFD8(dword_81184, k, 2, v37);
    v18 = sub_12F04(aN);
  }
  itow(a5, v37, 10);
  v19 = sub_12F04(aR);
  for ( m = sub_3B0C0(dword_81184, aR, 0, v19); m != -1; m = sub_3B0C0(dword_81184, aR, 0, v21) )
  {
    sub_3AFD8(dword_81184, m, 2, v37);
    v21 = sub_12F04(aR);
  }
  if ( a3 == 1 )
  {
    v22 = a4 + 288;
  }
  else
  {
    v36 = a3;
    v22 = *(_DWORD *)sub_1DD10((_DWORD *)(a4 + 496), &v36);
  }
  v23 = sub_16C9C((unsigned __int16 *)(v22 + 136), 2);
  itow(v23, v37, 10);
  v24 = sub_12F04(aM);
  for ( n = sub_3B0C0(dword_81184, aM, 0, v24); n != -1; n = sub_3B0C0(dword_81184, aM, 0, v26) )
  {
    sub_3AFD8(dword_81184, n, 2, v37);
    v26 = sub_12F04(aM);
  }
  v27 = sub_12F04(aC);
  for ( ii = sub_3B0C0(dword_81184, aC, 0, v27); ii != -1; ii = sub_3B0C0(dword_81184, aC, 0, v30) )
  {
    v29 = sub_173C4(a4 + 652, (unsigned __int8)a5);
    sub_3AF0C(dword_81184, ii, 2, v29 + 8);
    v30 = sub_12F04(aC);
  }
  v31 = sub_12F04(aG);
  result = sub_3B0C0(dword_81184, aG, 0, v31);
  for ( jj = result; result != -1; jj = result )
  {
    v34 = sub_3B874(a4 + 608, (unsigned __int8)a5);
    sub_3AF0C(dword_81184, jj, 2, v34 + 4);
    v35 = sub_12F04(aG);
    result = sub_3B0C0(dword_81184, aG, 0, v35);
  }
  return result;
}

