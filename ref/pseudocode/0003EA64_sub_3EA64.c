/* ea:   0x0003EA64 */
/* name: sub_3EA64 */
/* size: 96 */
int __fastcall sub_3EA64(int a1)
{
  int v2; // r4
  int v3; // r1

  v2 = 0;
  if ( (unsigned int)sub_5AE00(*(_DWORD *)(a1 + 20)) >= 0x19 )
    v3 = sub_5AE00(*(_DWORD *)(a1 + 20)) - 25;
  else
    v3 = 0;
  sub_5ADA4(*(_DWORD *)(a1 + 20), v3);
  if ( (unsigned int)sub_5AE68(*(_DWORD *)(a1 + 20)) >= 0x19 )
    v2 = sub_5AE68(*(_DWORD *)(a1 + 20)) - 25;
  return sub_5AE08(*(_DWORD *)(a1 + 20), v2);
}

