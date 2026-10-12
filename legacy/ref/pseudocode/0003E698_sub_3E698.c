/* ea:   0x0003E698 */
/* name: sub_3E698 */
/* size: 136 */
int __fastcall sub_3E698(int a1)
{
  int v2; // r0
  void *v4; // [sp+8h] [bp-10h] BYREF

  if ( operator new(0x84u) != nullptr )
    v2 = sub_59060();
  else
    v2 = 0;
  v4 = nullptr;
  sub_3E720(a1 + 20, v2);
  sub_3DFCC(&v4);
  if ( *(_DWORD *)(a1 + 20) != 0 )
    return sub_590A8(*(_DWORD *)(a1 + 20), 22050, 8, 0, 1, 8);
  else
    return 0;
}

