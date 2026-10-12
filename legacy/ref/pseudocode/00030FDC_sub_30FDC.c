/* ea:   0x00030FDC */
/* name: sub_30FDC */
/* size: 224 */
void __fastcall sub_30FDC(int a1)
{
  HANDLE FileW; // r0
  void *v3; // r4
  DWORD NumberOfBytesWritten; // [sp+Ch] [bp-24h] BYREF
  int Buffer; // [sp+10h] [bp-20h] BYREF
  LPCWSTR lpFileName[7]; // [sp+14h] [bp-1Ch] BYREF

  sub_1333C(lpFileName);
  sub_1309C((int)lpFileName);
  sub_162DC(lpFileName, (char *)aDataSettingsDa);
  FileW = CreateFileW(lpFileName[0], 0x40000000u, 0, nullptr, 2u, 0x80u, nullptr);
  v3 = FileW;
  if ( FileW != (HANDLE)-1 )
  {
    SetFilePointer(FileW, 2, nullptr, 0);
    Buffer = 19810929;
    WriteFile(v3, &Buffer, 4u, &NumberOfBytesWritten, nullptr);
    WriteFile(v3, (LPCVOID)(a1 + 32), 1u, &NumberOfBytesWritten, nullptr);
    WriteFile(v3, (LPCVOID)(a1 + 33), 1u, &NumberOfBytesWritten, nullptr);
    CloseHandle(v3);
  }
  sub_12F48((void **)lpFileName);
}

