/* ea:   0x00052EB4 */
/* name: sub_52EB4 */
/* size: 100 */
int __fastcall sub_52EB4(char *lpBuffer, HANDLE hFile)
{
  DWORD NumberOfBytesWritten; // [sp+4h] [bp-14h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  WriteFile(hFile, lpBuffer, 2u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, lpBuffer + 2, 2u, &NumberOfBytesWritten, nullptr);
  return 1;
}

