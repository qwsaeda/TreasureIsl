/* ea:   0x000612D8 */
/* name: sub_612D8 */
/* size: 84 */
BOOL __fastcall sub_612D8(HANDLE *a1, LONG a2, DWORD a3)
{
  if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))*a1 + 9))(a1) == 0 )
    __debugbreak();
  return SetFilePointer(a1[1], a2, nullptr, a3) != -1;
}

