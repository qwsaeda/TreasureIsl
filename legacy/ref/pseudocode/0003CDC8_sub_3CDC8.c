/* ea:   0x0003CDC8 */
/* name: sub_3CDC8 */
/* size: 20 */
void __fastcall sub_3CDC8(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

