/* ea:   0x0002F1E4 */
/* name: sub_2F1E4 */
/* size: 632 */
int __fastcall sub_2F1E4(int a1, _DWORD *a2, int a3)
{
  _DWORD *v4; // r9
  int i; // r7
  __int16 **v7; // r0
  HANDLE FileW; // r0
  void *v9; // r6
  _DWORD *v10; // r2
  int j; // r3
  int v12; // r3
  int v13; // r0
  DWORD NumberOfBytesRead; // [sp+10h] [bp-8Ch] BYREF
  int Buffer; // [sp+14h] [bp-88h] BYREF
  int v17; // [sp+18h] [bp-84h] BYREF
  int v18; // [sp+1Ch] [bp-80h] BYREF
  wchar_t v19[2]; // [sp+24h] [bp-78h] BYREF
  int v20; // [sp+28h] [bp-74h]
  LPCWSTR lpFileName[4]; // [sp+2Ch] [bp-70h] BYREF
  _DWORD v22[4]; // [sp+3Ch] [bp-60h] BYREF
  void *v23[4]; // [sp+4Ch] [bp-50h] BYREF
  void *v24[4]; // [sp+5Ch] [bp-40h] BYREF
  void *v25[12]; // [sp+6Ch] [bp-30h] BYREF

  v4 = a2;
  memset(a2, 0, 16 * a3);
  sub_1333C(v22);
  sub_1333C(v25);
  sub_1333C(v24);
  sub_1333C(lpFileName);
  sub_1309C((int)v22);
  sub_162DC(v22, (char *)aDataGl);
  for ( i = 0; i < a3; v4 += 4 )
  {
    *(_DWORD *)v19 = 0;
    v20 = 0;
    itow(i, v19, 10);
    v7 = (__int16 **)sub_37EDC(v23, v22, v19);
    if ( v7 != (__int16 **)lpFileName )
      sub_17890((int)lpFileName, *v7, v7[1]);
    sub_12F48(v23);
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
        v17 = 0;
        v18 = 0;
        ReadFile(v9, &v17, 4u, &NumberOfBytesRead, nullptr);
        v10 = *(_DWORD **)(a1 + 264);
        for ( j = 0; j < 5; ++j )
        {
          if ( *v10 == v17 )
            break;
          ++v10;
        }
        *v4 = *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4 * j + 44);
        SetFilePointer(v9, 4, nullptr, 1u);
        ReadFile(v9, v4 + 3, 4u, &NumberOfBytesRead, nullptr);
        ReadFile(v9, &v18, 4u, &NumberOfBytesRead, nullptr);
        v12 = v18;
        v13 = 30 * (v18 / 30);
        v4[1] = v18 / 30 + 1;
        v4[2] = v12 - v13 + 1;
      }
      CloseHandle(v9);
    }
    ++i;
  }
  sub_12F48((void **)lpFileName);
  sub_12F48(v24);
  sub_12F48(v25);
  return sub_12EA0((int)v22);
}

