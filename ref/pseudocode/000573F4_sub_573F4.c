/* ea:   0x000573F4 */
/* name: sub_573F4 */
/* size: 92 */
int __fastcall sub_573F4(int a1)
{
  if ( *(_DWORD *)(a1 + 1080) == 1 )
  {
    *(_DWORD *)(*(_DWORD *)(a1 + 1116) + 12) = 60;
    *(_DWORD *)(*(_DWORD *)(a1 + 1116) + 16) = 95;
    return sub_460F8(*(_DWORD **)(a1 + 1116), *(int **)(a1 + 4), *(int (**)())(a1 + 488));
  }
  else
  {
    *(_DWORD *)(*(_DWORD *)(a1 + 1120) + 12) = 60;
    *(_DWORD *)(*(_DWORD *)(a1 + 1120) + 16) = 95;
    return sub_460F8(*(_DWORD **)(a1 + 1120), *(int **)(a1 + 4), *(int (**)())(a1 + 488));
  }
}

