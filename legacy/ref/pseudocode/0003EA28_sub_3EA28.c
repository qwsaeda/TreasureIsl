/* ea:   0x0003EA28 */
/* name: sub_3EA28 */
/* size: 60 */
int __fastcall sub_3EA28(int a1, unsigned int a2)
{
  int v4; // r0
  int v5; // r5
  bool v6; // cc
  int v7; // r1

  v4 = *(_DWORD *)(a1 + 20);
  v5 = 255;
  v6 = a2 > 0xFF;
  v7 = 255;
  if ( !v6 )
    v7 = a2;
  sub_5ADA4(v4, v7);
  if ( a2 <= 0xFF )
    v5 = a2;
  return sub_5AE08(*(_DWORD *)(a1 + 20), v5);
}

