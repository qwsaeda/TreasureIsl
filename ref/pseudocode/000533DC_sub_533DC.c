/* ea:   0x000533DC */
/* name: sub_533DC */
/* size: 100 */
int __fastcall sub_533DC(char *lpBuffer, HANDLE hFile)
{
  DWORD NumberOfBytesRead; // [sp+4h] [bp-14h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  ReadFile(hFile, lpBuffer, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, lpBuffer + 4, 2u, &NumberOfBytesRead, nullptr);
  return 1;
}

