/* ea:   0x00063DD4 */
/* name: sub_63DD4 */
/* size: 844 */
int __fastcall sub_63DD4(int a1)
{
  int v2; // r10
  unsigned __int8 *v3; // r0
  unsigned __int8 *v4; // r6
  unsigned __int8 *v5; // r8
  int v6; // r1
  unsigned int v7; // r7
  int v8; // r6
  int v9; // r5
  int v10; // r0
  int v11; // r3
  void *v13; // r0
  void *v14; // r9
  unsigned __int8 *v15; // r7
  unsigned int v16; // r8
  int v17; // r6
  int v18; // r5
  int v19; // r0
  int v20; // r3
  unsigned __int8 *v21; // [sp+0h] [bp-4Ch]
  int v22; // [sp+4h] [bp-48h] BYREF
  _DWORD v23[4]; // [sp+8h] [bp-44h] BYREF
  _DWORD v24[13]; // [sp+18h] [bp-34h] BYREF

  v2 = a1 + 20;
  v3 = (unsigned __int8 *)(*(int (__fastcall **)(int))(*(_DWORD *)(a1 + 20) + 32))(a1 + 20);
  if ( v3 == nullptr )
    __debugbreak();
  v4 = v3 + 38;
  v5 = v3 + 54;
  v21 = v3 + 38;
  sub_64434(a1 + 8, v3[38] | (v3[39] << 8) | (v3[40] << 16) | (v3[41] << 24));
  v6 = *v4 | (v4[1] << 8) | (v4[2] << 16) | (v4[3] << 24);
  if ( (v4[4] | (v4[5] << 8) | (v4[6] << 16) | (v4[7] << 24)) != 0 )
  {
    v22 = 16 * v6;
    v13 = malloc(16 * v6);
    v14 = v13;
    if ( v13 == nullptr )
      return 0;
    if ( sub_66B04(v13, &v22, v4 + 16) != 0 )
    {
      free(v14);
      return 0;
    }
    v15 = (unsigned __int8 *)v14;
    v16 = 0;
    if ( (*v4 | (v4[1] << 8) | (v4[2] << 16) | (v4[3] << 24)) != 0 )
    {
      do
      {
        v17 = (v4[12] | (v4[13] << 8) | (v4[14] << 16) | (v4[15] << 24))
            + (v15[4] | (v15[5] << 8) | (v15[6] << 16) | (v15[7] << 24))
            + (*(int (__fastcall **)(int))(*(_DWORD *)(a1 + 20) + 32))(v2);
        v18 = v15[12] | (v15[13] << 8) | (v15[14] << 16) | (v15[15] << 24);
        v19 = v15[3];
        v20 = *v15 | (v15[1] << 8) | (v15[2] << 16);
        v24[1] = v15[8] | (v15[9] << 8) | (v15[10] << 16) | (v15[11] << 24);
        v24[2] = v18;
        v24[0] = v20 | (v19 << 24);
        v24[3] = v17;
        sub_64120(a1 + 8, v24);
        v4 = v21;
        ++v16;
        v15 += 16;
      }
      while ( v16 < (*v21 | (v21[1] << 8) | (v21[2] << 16) | (v21[3] << 24)) );
    }
    free(v14);
    return 1;
  }
  v7 = 0;
  if ( v6 == 0 )
    return 1;
  do
  {
    v8 = (v5[4] | (v5[5] << 8) | (v5[6] << 16) | (v5[7] << 24))
       + (v4[12] | (v4[13] << 8) | (v4[14] << 16) | (v4[15] << 24))
       + (*(int (__fastcall **)(int))(*(_DWORD *)(a1 + 20) + 32))(v2);
    v9 = v5[12] | (v5[13] << 8) | (v5[14] << 16) | (v5[15] << 24);
    v10 = v5[3];
    v11 = *v5 | (v5[1] << 8) | (v5[2] << 16);
    v23[1] = v5[8] | (v5[9] << 8) | (v5[10] << 16) | (v5[11] << 24);
    v23[2] = v9;
    v23[0] = v11 | (v10 << 24);
    v23[3] = v8;
    sub_64120(a1 + 8, v23);
    v4 = v21;
    ++v7;
    v5 += 16;
  }
  while ( v7 < (*v21 | (v21[1] << 8) | (v21[2] << 16) | (v21[3] << 24)) );
  return 1;
}

