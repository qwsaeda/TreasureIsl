/* ea:   0x00041E34 */
/* name: sub_41E34 */
/* size: 588 */
int __fastcall sub_41E34(int a1, _DWORD *a2, int a3)
{
  _DWORD *v4; // r9
  int i; // r7
  __int16 **v7; // r0
  HANDLE FileW; // r0
  void *v9; // r6
  int v10; // r3
  int v11; // r0
  DWORD NumberOfBytesRead; // [sp+10h] [bp-8Ch] BYREF
  int Buffer; // [sp+14h] [bp-88h] BYREF
  int v15; // [sp+18h] [bp-84h] BYREF
  int v16; // [sp+1Ch] [bp-80h] BYREF
  wchar_t v17[2]; // [sp+24h] [bp-78h] BYREF
  int v18; // [sp+28h] [bp-74h]
  LPCWSTR lpFileName[4]; // [sp+2Ch] [bp-70h] BYREF
  _DWORD v20[4]; // [sp+3Ch] [bp-60h] BYREF
  void *v21[4]; // [sp+4Ch] [bp-50h] BYREF
  void *v22[4]; // [sp+5Ch] [bp-40h] BYREF
  _DWORD v23[12]; // [sp+6Ch] [bp-30h] BYREF

  v4 = a2;
  memset(a2, 0, 16 * a3);
  sub_1333C(v20);
  sub_1333C(v23);
  sub_1333C(v22);
  sub_1333C(lpFileName);
  sub_1309C((int)v20);
  sub_162DC(v20, (char *)aDataGl);
  for ( i = 0; i < a3; v4 += 4 )
  {
    *(_DWORD *)v17 = 0;
    v18 = 0;
    itow(i, v17, 10);
    v7 = (__int16 **)sub_37EDC(v21, (int)v20, (char *)v17);
    if ( v7 != (__int16 **)lpFileName )
      sub_17890((int)lpFileName, *v7, v7[1]);
    sub_12F48(v21);
    sub_162DC(lpFileName, (char *)aSav);
    FileW = CreateFileW(lpFileName[0], 0x80000000, 0, nullptr, 3u, 0x80u, nullptr);
    v9 = FileW;
    if ( FileW != (HANDLE)-1 )
    {
      SetFilePointer(FileW, 2, nullptr, 0);
      Buffer = 0;
      ReadFile(v9, &Buffer, 4u, &NumberOfBytesRead, nullptr);
      if ( Buffer == 19810929 )
      {
        v15 = 0;
        v16 = 0;
        ReadFile(v9, &v15, 4u, &NumberOfBytesRead, nullptr);
        *v4 = *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4 * v15 + 4);
        SetFilePointer(v9, 4, nullptr, 1u);
        ReadFile(v9, v4 + 3, 4u, &NumberOfBytesRead, nullptr);
        ReadFile(v9, &v16, 4u, &NumberOfBytesRead, nullptr);
        v10 = v16;
        v11 = 30 * (v16 / 30);
        v4[1] = v16 / 30 + 1;
        v4[2] = v10 - v11 + 1;
      }
      CloseHandle(v9);
    }
    ++i;
  }
  sub_12F48((void **)lpFileName);
  sub_12F48(v22);
  sub_12EA0((int)v23);
  return sub_12EA0((int)v20);
}

