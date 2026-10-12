/* ea:   0x0006132C */
/* name: sub_6132C */
/* size: 60 */
DWORD __fastcall sub_6132C(HANDLE *a1)
{
  if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))*a1 + 9))(a1) == 0 )
    __debugbreak();
  return SetFilePointer(a1[1], 0, nullptr, 1u);
}

