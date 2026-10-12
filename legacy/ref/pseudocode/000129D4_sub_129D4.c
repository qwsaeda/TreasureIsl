/* ea:   0x000129D4 */
/* name: sub_129D4 */
/* size: 636 */
int __fastcall sub_129D4(int a1)
{
  unsigned int v2; // r0
  int v3; // r6
  int *v4; // r4
  bool v5; // cc
  int v6; // r0
  int *v7; // r5
  int v8; // r3
  int v9; // r0
  bool v10; // zf
  int v11; // r0
  int v12; // r1
  int v13; // r0
  int v14; // r1
  int v15; // r1
  int v16; // r1
  int v17; // r1
  int v18; // r1
  int v19; // r1
  int v20; // r1
  int result; // r0
  int v22; // r0
  int v23; // r1

  v2 = *(unsigned __int8 *)(a1 + 1050);
  v3 = 0;
  v4 = *(int **)(a1 + 1036);
  *(_DWORD *)(a1 + 1108) = 0;
  *(_DWORD *)(a1 + 1112) = 0;
  if ( v2 > 1 )
  {
    *(_DWORD *)(a1 + 1108) = 1;
    *(_DWORD *)(a1 + 1112) = 1;
  }
  *(_DWORD *)(a1 + 1116) = 0;
  *(_DWORD *)(a1 + 1120) = 0;
  if ( v2 > 2 )
  {
    *(_DWORD *)(a1 + 1116) = 1;
    *(_DWORD *)(a1 + 1120) = 1;
  }
  v5 = v2 > 3;
  v6 = *(_DWORD *)(a1 + 1020);
  *(_DWORD *)(a1 + 1124) = 0;
  v7 = *(int **)(a1 + 1040);
  *(_DWORD *)(a1 + 1092) = 1;
  *(_DWORD *)(a1 + 1096) = 1;
  *(_DWORD *)(a1 + 1100) = 1;
  *(_DWORD *)(a1 + 1104) = 1;
  if ( v5 )
    *(_DWORD *)(a1 + 1124) = 1;
  v8 = sub_445F4(v6, 1);
  for ( *(_DWORD *)(v8 + 20) = 0; v4 != v7; ++v4 )
  {
    if ( *v4 == 4 )
      *(_DWORD *)(v8 + 20) = 1;
    v9 = *v4;
    if ( *v4 != 0 )
    {
      v10 = v9 == 1;
      if ( v9 != 1 )
        v10 = v9 == 3;
      if ( !v10 )
        *(_DWORD *)(a1 + 4 * *v4 + 1088) = 0;
    }
  }
  v11 = a1;
  v12 = 9;
  do
  {
    --v12;
    *(_DWORD *)(v11 + 1136) = 0;
    *(_DWORD *)(v11 + 1172) = 0;
    v11 += 4;
  }
  while ( v12 != 0 );
  *(_DWORD *)(a1 + 1256) = 0;
  if ( *(_DWORD *)(a1 + 1092) != 0 )
  {
    v13 = *(_DWORD *)(a1 + 380);
    v3 = 1;
    *(_DWORD *)(a1 + 1172) = 1;
    *(_DWORD *)(a1 + 1136) = v13;
    ++*(_DWORD *)(a1 + 1256);
  }
  if ( *(_DWORD *)(a1 + 1096) != 0 )
  {
    v14 = a1 + 4 * v3;
    *(_DWORD *)(v14 + 1136) = *(_DWORD *)(a1 + 384);
    ++v3;
    *(_DWORD *)(v14 + 1172) = 2;
    ++*(_DWORD *)(a1 + 1256);
  }
  if ( *(_DWORD *)(a1 + 1100) != 0 )
  {
    v15 = a1 + 4 * v3;
    *(_DWORD *)(v15 + 1136) = *(_DWORD *)(a1 + 388);
    ++v3;
    *(_DWORD *)(v15 + 1172) = 3;
    ++*(_DWORD *)(a1 + 1256);
  }
  if ( *(_DWORD *)(a1 + 1104) != 0 )
  {
    v16 = a1 + 4 * v3;
    *(_DWORD *)(v16 + 1136) = *(_DWORD *)(a1 + 392);
    ++v3;
    *(_DWORD *)(v16 + 1172) = 4;
    ++*(_DWORD *)(a1 + 1256);
  }
  if ( *(_DWORD *)(a1 + 1108) != 0 )
  {
    v17 = a1 + 4 * v3;
    *(_DWORD *)(v17 + 1136) = *(_DWORD *)(a1 + 396);
    ++v3;
    *(_DWORD *)(v17 + 1172) = 5;
    ++*(_DWORD *)(a1 + 1256);
  }
  if ( *(_DWORD *)(a1 + 1112) != 0 )
  {
    v18 = a1 + 4 * v3;
    *(_DWORD *)(v18 + 1136) = *(_DWORD *)(a1 + 400);
    ++v3;
    *(_DWORD *)(v18 + 1172) = 6;
    ++*(_DWORD *)(a1 + 1256);
  }
  if ( *(_DWORD *)(a1 + 1116) != 0 )
  {
    v19 = a1 + 4 * v3;
    *(_DWORD *)(v19 + 1136) = *(_DWORD *)(a1 + 404);
    ++v3;
    *(_DWORD *)(v19 + 1172) = 7;
    ++*(_DWORD *)(a1 + 1256);
  }
  if ( *(_DWORD *)(a1 + 1120) != 0 )
  {
    v20 = a1 + 4 * v3;
    *(_DWORD *)(v20 + 1136) = *(_DWORD *)(a1 + 408);
    ++v3;
    *(_DWORD *)(v20 + 1172) = 8;
    ++*(_DWORD *)(a1 + 1256);
  }
  result = *(_DWORD *)(a1 + 1124);
  if ( result != 0 )
  {
    v22 = *(_DWORD *)(a1 + 412);
    v23 = a1 + 4 * v3;
    *(_DWORD *)(v23 + 1172) = 9;
    *(_DWORD *)(v23 + 1136) = v22;
    result = *(_DWORD *)(a1 + 1256);
    *(_DWORD *)(a1 + 1256) = result + 1;
  }
  return result;
}

