/* ea:   0x0003EAC4 */
/* name: sub_3EAC4 */
/* size: 96 */
int __fastcall sub_3EAC4(int a1)
{
  int v2; // r4
  int v3; // r1

  v2 = 255;
  if ( (unsigned int)sub_5AE00(*(_DWORD *)(a1 + 20)) <= 0xE6 )
    v3 = sub_5AE00(*(_DWORD *)(a1 + 20)) + 25;
  else
    v3 = 255;
  sub_5ADA4(*(_DWORD *)(a1 + 20), v3);
  if ( (unsigned int)sub_5AE68(*(_DWORD *)(a1 + 20)) <= 0xE6 )
    v2 = sub_5AE68(*(_DWORD *)(a1 + 20)) + 25;
  return sub_5AE08(*(_DWORD *)(a1 + 20), v2);
}

