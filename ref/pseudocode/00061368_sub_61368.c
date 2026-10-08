/* ea:   0x00061368 */
/* name: sub_61368 */
/* size: 52 */
DWORD __fastcall sub_61368(HANDLE *a1)
{
  if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))*a1 + 9))(a1) == 0 )
    __debugbreak();
  return GetFileSize(a1[1], nullptr);
}

