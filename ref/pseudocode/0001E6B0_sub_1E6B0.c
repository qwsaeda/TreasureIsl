/* ea:   0x0001E6B0 */
/* name: sub_1E6B0 */
/* size: 44 */
int __fastcall sub_1E6B0(_DWORD *a1)
{
  bool v2; // zf
  int result; // r0

  v2 = (*(unsigned __int8 (__fastcall **)(_DWORD *))(*a1 + 36))(a1) == 0;
  result = a1[5];
  if ( v2 )
    __debugbreak();
  return result;
}

