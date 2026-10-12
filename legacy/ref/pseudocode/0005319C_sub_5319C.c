/* ea:   0x0005319C */
/* name: sub_5319C */
/* size: 100 */
int __fastcall sub_5319C(char *lpBuffer, HANDLE hFile)
{
  DWORD NumberOfBytesRead; // [sp+4h] [bp-14h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  ReadFile(hFile, lpBuffer, 2u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, lpBuffer + 2, 2u, &NumberOfBytesRead, nullptr);
  return 1;
}

