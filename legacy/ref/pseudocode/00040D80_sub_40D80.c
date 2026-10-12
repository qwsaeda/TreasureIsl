/* ea:   0x00040D80 */
/* name: sub_40D80 */
/* size: 816 */
int __fastcall sub_40D80(_DWORD *a1, int a2, int a3)
{
  __int16 **v5; // r0
  __int16 **v6; // r0
  void *v7; // r0
  int v8; // r3
  _DWORD *v9; // r4
  _DWORD *v10; // r0
  _DWORD *v11; // r0
  HANDLE FileW; // r0
  void *v13; // r5
  DWORD NumberOfBytesRead; // [sp+10h] [bp-9Ch] BYREF
  int Buffer; // [sp+14h] [bp-98h] BYREF
  void *v16[4]; // [sp+1Ch] [bp-90h] BYREF
  void *v17[4]; // [sp+2Ch] [bp-80h] BYREF
  void *v18[4]; // [sp+3Ch] [bp-70h] BYREF
  LPCWSTR lpFileName[4]; // [sp+4Ch] [bp-60h] BYREF
  void *v20[4]; // [sp+5Ch] [bp-50h] BYREF
  void *v21[4]; // [sp+6Ch] [bp-40h] BYREF
  _DWORD v22[4]; // [sp+7Ch] [bp-30h] BYREF
  int v23; // [sp+8Ch] [bp-20h] BYREF

  a1[1] = a2;
  if ( a2 == 0 )
    return 0;
  a1[2] = a3;
  if ( a3 == 0 )
    return 0;
  sub_1333C(v17);
  sub_1333C(v16);
  sub_1333C(v18);
  sub_1309C((int)v17);
  sub_162DC(v17, (char *)aData);
  v5 = (__int16 **)sub_37EDC(v20, (int)v17, (char *)aUiPak);
  if ( v5 != (__int16 **)v16 )
    sub_17890((int)v16, *v5, v5[1]);
  sub_12F48(v20);
  v6 = (__int16 **)sub_37EDC(v21, (int)v17, (char *)aIndexMap);
  if ( v6 != (__int16 **)v18 )
    sub_17890((int)v18, *v6, v6[1]);
  sub_12F48(v21);
  v7 = operator new(0x3FCu);
  if ( v7 != nullptr )
    v8 = sub_43B8C(v7, a1[2]);
  else
    v8 = 0;
  a1[12] = v8;
  if ( v8 != 0 )
  {
    sub_43D54(v8, *(_DWORD *)(a1[1] + 16));
    v9 = sub_132FC(v22, v18);
    v10 = sub_132FC(&v23, v16);
    (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)a1[12] + 4))(
      a1[12],
      *v10,
      v10[1],
      v10[2],
      *v9,
      v9[1],
      v9[2]);
    v11 = sub_37EDC(v22, (int)v17, (char *)aGlpicsPak);
    sub_161B0((int)v16, (int)v11);
    sub_12EA0((int)v22);
    if ( (unsigned __int8)sub_41B5C(a1, v16[0], a1 + 8) != 0 )
    {
      sub_1333C(lpFileName);
      sub_1309C((int)lpFileName);
      sub_162DC(lpFileName, (char *)aDataGlRec);
      FileW = CreateFileW(lpFileName[0], 0x80000000, 0, nullptr, 3u, 0x80u, nullptr);
      v13 = FileW;
      if ( FileW != (HANDLE)-1 )
      {
        SetFilePointer(FileW, 2, nullptr, 0);
        Buffer = 0;
        ReadFile(v13, &Buffer, 4u, &NumberOfBytesRead, nullptr);
        if ( Buffer == 19810929 )
        {
          ReadFile(v13, &byte_81194, 1u, &NumberOfBytesRead, nullptr);
          ReadFile(v13, &byte_81195, 1u, &NumberOfBytesRead, nullptr);
          ReadFile(v13, &byte_81196, 1u, &NumberOfBytesRead, nullptr);
          ReadFile(v13, &byte_81197, 1u, &NumberOfBytesRead, nullptr);
        }
        CloseHandle(v13);
      }
      off_81198 = nullptr;
      off_8119C = nullptr;
      sub_12EA0((int)lpFileName);
      sub_12EA0((int)v18);
      sub_12EA0((int)v16);
      sub_12EA0((int)v17);
      return 1;
    }
    else
    {
      sub_12EA0((int)v18);
      sub_12EA0((int)v16);
      sub_12EA0((int)v17);
      return 0;
    }
  }
  else
  {
    sub_12F48(v18);
    sub_12F48(v16);
    sub_12F48(v17);
    return 0;
  }
}

