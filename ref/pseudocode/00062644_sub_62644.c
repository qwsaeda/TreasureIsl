/* ea:   0x00062644 */
/* name: sub_62644 */
/* size: 40 */
int __fastcall sub_62644(int a1)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 8);
  if ( result != 0 )
    return (*(int (__fastcall **)(int))(*(_DWORD *)result + 20))(result);
  return result;
}

