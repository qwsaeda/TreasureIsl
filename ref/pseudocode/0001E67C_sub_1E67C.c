/* ea:   0x0001E67C */
/* name: sub_1E67C */
/* size: 52 */
DWORD __fastcall sub_1E67C(HANDLE *a1)
{
  if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))*a1 + 9))(a1) == 0 )
    __debugbreak();
  return GetFileSize(a1[1], nullptr);
}

