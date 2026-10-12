/* ea:   0x000402D8 */
/* name: sub_402D8 */
/* size: 1028 */
int __fastcall sub_402D8(_DWORD *a1, int a2, int a3)
{
  __int16 **v5; // r0
  __int16 **v6; // r0
  void *v7; // r0
  int v8; // r3
  _DWORD *v9; // r4
  _DWORD *v10; // r0
  __int16 **v11; // r0
  HANDLE FileW; // r4
  _DWORD *v13; // r7
  _DWORD *v14; // r6
  _DWORD *v15; // r5
  _DWORD *v16; // r0
  HANDLE v17; // r4
  _DWORD *v18; // r0
  HANDLE v19; // r4
  DWORD Buffer; // [sp+10h] [bp-ACh] BYREF
  int v21; // [sp+14h] [bp-A8h] BYREF
  int v22; // [sp+18h] [bp-A4h] BYREF
  LPCWSTR lpFileName[4]; // [sp+1Ch] [bp-A0h] BYREF
  void *v24[4]; // [sp+2Ch] [bp-90h] BYREF
  void *v25[4]; // [sp+3Ch] [bp-80h] BYREF
  void *v26[4]; // [sp+4Ch] [bp-70h] BYREF
  void *v27[4]; // [sp+5Ch] [bp-60h] BYREF
  void *v28[4]; // [sp+6Ch] [bp-50h] BYREF
  _DWORD v29[4]; // [sp+7Ch] [bp-40h] BYREF
  int v30; // [sp+8Ch] [bp-30h] BYREF

  a1[1] = a2;
  if ( a2 == 0 )
    return 0;
  a1[2] = a3;
  if ( a3 == 0 )
    return 0;
  sub_1333C(v24);
  sub_1333C(v25);
  sub_1333C(lpFileName);
  sub_1309C((int)v24);
  sub_162DC(v24, (char *)aData);
  v5 = (__int16 **)sub_37EDC(v26, (int)v24, (char *)aUiPak);
  if ( v5 != (__int16 **)v25 )
    sub_17890((int)v25, *v5, v5[1]);
  sub_12F48(v26);
  v6 = (__int16 **)sub_37EDC(v27, (int)v24, (char *)aSelectmapMap);
  if ( v6 != (__int16 **)lpFileName )
    sub_17890((int)lpFileName, *v6, v6[1]);
  sub_12F48(v27);
  v7 = operator new(0x3FCu);
  if ( v7 != nullptr )
    v8 = sub_43B8C(v7, a1[2]);
  else
    v8 = 0;
  a1[12] = v8;
  if ( v8 != 0 )
  {
    sub_43D54(v8, *(_DWORD *)(a1[1] + 16));
    v9 = sub_132FC(v29, lpFileName);
    v10 = sub_132FC(&v30, v25);
    (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)a1[12] + 4))(
      a1[12],
      *v10,
      v10[1],
      v10[2],
      *v9,
      v9[1],
      v9[2]);
    Buffer = 0;
    v11 = (__int16 **)sub_37EDC(v28, (int)v24, (char *)aGlmapm01Map);
    if ( v11 != (__int16 **)lpFileName )
      sub_17890((int)lpFileName, *v11, v11[1]);
    sub_12F48(v28);
    FileW = CreateFileW(lpFileName[0], 0x80000000, 1u, nullptr, 3u, 0x80u, nullptr);
    if ( ReadFile(FileW, &Buffer, 4u, &Buffer, nullptr) )
    {
      v13 = a1 + 13;
      v21 = 1;
      sub_406DC(a1 + 13, &v21);
      v14 = a1 + 16;
      v22 = 73;
      sub_406DC(a1 + 16, &v22);
      v15 = a1 + 19;
      v22 = 100;
      sub_406DC(a1 + 19, &v22);
      CloseHandle(FileW);
      v16 = sub_37EDC(v29, (int)v24, (char *)aGlmapm02Map);
      sub_161B0((int)lpFileName, (int)v16);
      sub_12EA0((int)v29);
      v17 = CreateFileW(lpFileName[0], 0x80000000, 1u, nullptr, 3u, 0x80u, nullptr);
      if ( ReadFile(v17, &Buffer, 4u, &Buffer, nullptr) )
      {
        v22 = 2;
        sub_406DC(v13, &v22);
        v22 = 67;
        sub_406DC(v14, &v22);
        v22 = 64;
        sub_406DC(v15, &v22);
      }
      CloseHandle(v17);
      v18 = sub_37EDC(v29, (int)v24, (char *)aGlmapm03Map);
      sub_161B0((int)lpFileName, (int)v18);
      sub_12EA0((int)v29);
      v19 = CreateFileW(lpFileName[0], 0x80000000, 1u, nullptr, 3u, 0x80u, nullptr);
      if ( ReadFile(v19, &Buffer, 4u, &Buffer, nullptr) )
      {
        v22 = 3;
        sub_406DC(v13, &v22);
        v22 = 117;
        sub_406DC(v14, &v22);
        v22 = 81;
        sub_406DC(v15, &v22);
      }
      CloseHandle(v19);
      sub_12EA0((int)lpFileName);
      sub_12EA0((int)v25);
      sub_12EA0((int)v24);
      return 1;
    }
    else
    {
      sub_12EA0((int)lpFileName);
      sub_12EA0((int)v25);
      sub_12EA0((int)v24);
      return 0;
    }
  }
  else
  {
    sub_12F48((void **)lpFileName);
    sub_12F48(v25);
    sub_12F48(v24);
    return 0;
  }
}

