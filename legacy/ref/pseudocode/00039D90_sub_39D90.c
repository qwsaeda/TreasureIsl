/* ea:   0x00039D90 */
/* name: sub_39D90 */
/* size: 76 */
int __fastcall sub_39D90(int result, int a2, int a3)
{
  int v3; // r3
  int v5; // r2

  v3 = result;
  if ( result != a3 )
  {
    **(_DWORD **)(a3 + 4) = result;
    **(_DWORD **)(a2 + 4) = a3;
    **(_DWORD **)(result + 4) = a2;
    result = *(_DWORD *)(a3 + 4);
    v5 = *(_DWORD *)(v3 + 4);
    *(_DWORD *)(v3 + 4) = result;
    *(_DWORD *)(a3 + 4) = *(_DWORD *)(a2 + 4);
    *(_DWORD *)(a2 + 4) = v5;
  }
  return result;
}

