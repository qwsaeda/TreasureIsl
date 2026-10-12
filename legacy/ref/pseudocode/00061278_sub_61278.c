/* ea:   0x00061278 */
/* name: sub_61278 */
/* size: 96 */
DWORD __fastcall sub_61278(HANDLE *a1, const void *a2, DWORD a3)
{
  bool v6; // zf
  DWORD result; // r0
  DWORD NumberOfBytesWritten; // [sp+4h] [bp-14h] BYREF

  if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))*a1 + 9))(a1) == 0 )
    __debugbreak();
  v6 = !WriteFile(a1[1], a2, a3, &NumberOfBytesWritten, nullptr);
  result = -1;
  if ( !v6 )
    return NumberOfBytesWritten;
  return result;
}

