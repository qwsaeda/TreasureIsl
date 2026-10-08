/* ea:   0x00062544 */
/* name: sub_62544 */
/* size: 52 */
int __fastcall sub_62544(int a1)
{
  if ( *(_BYTE *)(a1 + 4) == 0 )
    __debugbreak();
  return (*(int (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 8) + 16))(*(_DWORD *)(a1 + 8));
}

