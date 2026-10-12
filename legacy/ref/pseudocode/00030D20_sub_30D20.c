/* ea:   0x00030D20 */
/* name: sub_30D20 */
/* size: 20 */
void __fastcall sub_30D20(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

