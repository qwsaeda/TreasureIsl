/* ea:   0x0003EB24 */
/* name: sub_3EB24 */
/* size: 236 */
void __fastcall sub_3EB24(int a1)
{
  HANDLE FileW; // r0
  void *v3; // r4
  int Buffer; // [sp+Ch] [bp-24h] BYREF
  DWORD NumberOfBytesRead; // [sp+10h] [bp-20h] BYREF
  LPCWSTR lpFileName[7]; // [sp+14h] [bp-1Ch] BYREF

  sub_1333C(lpFileName);
  sub_1309C((int)lpFileName);
  sub_162DC(lpFileName, (char *)aDataSettingsDa);
  FileW = CreateFileW(lpFileName[0], 0x80000000, 0, nullptr, 3u, 0x80u, nullptr);
  v3 = FileW;
  if ( FileW != (HANDLE)-1 )
  {
    SetFilePointer(FileW, 2, nullptr, 0);
    Buffer = 0;
    ReadFile(v3, &Buffer, 4u, &NumberOfBytesRead, nullptr);
    if ( Buffer == 19810929 )
    {
      ReadFile(v3, (LPVOID)(a1 + 32), 1u, &NumberOfBytesRead, nullptr);
      ReadFile(v3, (LPVOID)(a1 + 33), 1u, &NumberOfBytesRead, nullptr);
    }
    CloseHandle(v3);
  }
  sub_12F48((void **)lpFileName);
}

