/* ea:   0x0005F138 */
/* name: sub_5F138 */
/* size: 100 */
int __fastcall sub_5F138(int a1, unsigned __int16 *a2, int a3)
{
  int v6; // r1
  bool v7; // zf

  if ( a2 == nullptr )
    __debugbreak();
  sub_196AC((_DWORD *)(a1 + 4));
  v6 = *a2;
  v7 = v6 == 65279;
  if ( v6 != 65279 )
    v7 = v6 == 65534;
  if ( v7 )
    return sub_5FBEC(a1, a2 + 1, a3 - 2);
  else
    return 0;
}

