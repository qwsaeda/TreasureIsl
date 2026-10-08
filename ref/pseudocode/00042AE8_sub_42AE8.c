/* ea:   0x00042AE8 */
/* name: sub_42AE8 */
/* size: 712 */
void __fastcall sub_42AE8(int a1)
{
  HANDLE FileW; // r0
  void *v3; // r4
  unsigned int v4; // r3
  HANDLE v5; // r4
  DWORD NumberOfBytesRead; // [sp+10h] [bp-48h] BYREF
  int v7; // [sp+14h] [bp-44h] BYREF
  int Buffer; // [sp+18h] [bp-40h] BYREF
  unsigned int v9; // [sp+1Ch] [bp-3Ch] BYREF
  LPCWSTR lpFileName[4]; // [sp+24h] [bp-34h] BYREF
  struct _SYSTEMTIME SystemTime; // [sp+34h] [bp-24h] BYREF

  sub_16480(lpFileName, word_7FA78);
  FileW = CreateFileW(lpFileName[0], 0xC0000000, 0, nullptr, 3u, 0x80u, nullptr);
  v3 = FileW;
  if ( FileW == (HANDLE)-1 )
  {
    v5 = CreateFileW(lpFileName[0], 0x40000000u, 0, nullptr, 2u, 0x80u, nullptr);
    if ( v5 != (HANDLE)-1 )
    {
      GetLocalTime(&SystemTime);
      Buffer = sub_42DB0(a1, &SystemTime);
      v7 = Buffer;
      WriteFile(v5, &Buffer, 4u, &NumberOfBytesRead, nullptr);
      WriteFile(v5, &v7, 4u, &NumberOfBytesRead, nullptr);
      CloseHandle(v5);
    }
    sub_12F48((void **)lpFileName);
  }
  else if ( GetFileSize(FileW, nullptr) != 0 )
  {
    GetLocalTime(&SystemTime);
    if ( ReadFile(v3, &Buffer, 4u, &NumberOfBytesRead, nullptr) && ReadFile(v3, &v9, 4u, &NumberOfBytesRead, nullptr) )
    {
      v4 = sub_42DB0(a1, &SystemTime);
      v7 = v4;
      if ( v4 <= Buffer + 2592000 && v4 > v9 && SystemTime.wYear == 2005 )
      {
        GetLocalTime(&SystemTime);
        v7 = sub_42DB0(a1, &SystemTime);
        SetFilePointer(v3, 4, nullptr, 0);
        WriteFile(v3, &v7, 4u, &NumberOfBytesRead, nullptr);
        CloseHandle(v3);
        sub_12F48((void **)lpFileName);
      }
      else
      {
        SetFilePointer(v3, 4, nullptr, 0);
        WriteFile(v3, &v7, 4u, &NumberOfBytesRead, nullptr);
        sub_42E74(a1, &unk_7FA40);
        CloseHandle(v3);
        sub_12F48((void **)lpFileName);
      }
    }
    else
    {
      sub_42E74(a1, &unk_7FA40);
      CloseHandle(v3);
      sub_132A0((void **)lpFileName);
    }
  }
  else
  {
    GetLocalTime(&SystemTime);
    Buffer = sub_42DB0(a1, &SystemTime);
    v7 = Buffer;
    WriteFile(v3, &Buffer, 4u, &NumberOfBytesRead, nullptr);
    WriteFile(v3, &v7, 4u, &NumberOfBytesRead, nullptr);
    CloseHandle(v3);
    sub_12F48((void **)lpFileName);
  }
}

