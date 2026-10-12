/* ea:   0x0003B874 */
/* name: sub_3B874 */
/* size: 76 */
int __fastcall sub_3B874(int a1, int a2)
{
  int result; // r0
  char v4; // [sp+0h] [bp-10h] BYREF

  result = sub_3B8C0(*(_DWORD *)(a1 + 16), *(_DWORD *)(a1 + 20), a2, &v4);
  if ( result == *(_DWORD *)(a1 + 20) )
    __debugbreak();
  return result;
}

