/* ea:   0x00053200 */
/* name: sub_53200 */
/* size: 100 */
int __fastcall sub_53200(char *lpBuffer, HANDLE hFile)
{
  DWORD NumberOfBytesRead; // [sp+4h] [bp-14h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  ReadFile(hFile, lpBuffer, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, lpBuffer + 4, 4u, &NumberOfBytesRead, nullptr);
  return 1;
}

